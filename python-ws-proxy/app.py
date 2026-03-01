#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
python-ws-proxy: WebSocket proxy service
Supports VLESS, Trojan, Shadowsocks protocols
"""

import os
import sys
import socket
import struct
import hashlib
import base64
import asyncio
import ipaddress
import logging
from aiohttp import web

# ============== Configuration ==============
UUID = os.environ.get('UUID', '5efabea4-f6d4-91fd-b8f0-17e004c89c60')
PORT = int(os.environ.get('SERVER_PORT') or os.environ.get('PORT') or 3000)
DOMAIN = os.environ.get('DOMAIN', '')
WS_PATH = os.environ.get('WS_PATH', UUID[:8])
SUB_PATH = os.environ.get('SUB_PATH', 'sub')
NAME = os.environ.get('NAME', '')
AUTO_ACCESS = os.environ.get('AUTO_ACCESS', '').lower() == 'true'
DEBUG = os.environ.get('DEBUG', '').lower() == 'true'

# ============== Global Variables ==============
CurrentDomain = DOMAIN
CurrentPort = 443
Tls = 'tls'
ISP = ''

# ============== Blocked Domains ==============
BLOCKED_DOMAINS = [
    'speedtest.net', 'fast.com', 'speedtest.cn', 'speed.cloudflare.com',
    'speedof.me', 'testmy.net', 'bandwidth.place', 'speed.io',
    'librespeed.org', 'speedcheck.org', 'nflxvideo.net', 'nflxso.net', 'nflxext.com'
]

# ============== Logging ==============
log_level = logging.DEBUG if DEBUG else logging.INFO
logging.basicConfig(
    level=log_level,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logging.getLogger('aiohttp.access').setLevel(logging.WARNING)
logging.getLogger('aiohttp.server').setLevel(logging.WARNING)
logger = logging.getLogger(__name__)


def is_blocked_domain(host: str) -> bool:
    """Check if domain is blocked"""
    if not host:
        return False
    host_lower = host.lower()
    return any(host_lower == blocked or host_lower.endswith('.' + blocked)
               for blocked in BLOCKED_DOMAINS)


async def resolve_host(host: str) -> str:
    """Resolve hostname to IP address"""
    try:
        ipaddress.ip_address(host)
        return host
    except:
        pass

    # Try DNS resolution
    try:
        import socket as sock
        result = sock.getaddrinfo(host, None)
        if result:
            return result[0][4][0]
    except:
        pass

    return host


async def get_isp():
    """Get ISP info from IP API"""
    global ISP
    try:
        import aiohttp
        async with aiohttp.ClientSession() as session:
            async with session.get('https://api.ip.sb/geoip', timeout=3) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    ISP = f"{data.get('country_code', '')}-{data.get('isp', '')}".replace(' ', '_')
                    return
    except:
        pass

    try:
        import aiohttp
        async with aiohttp.ClientSession() as session:
            async with session.get('http://ip-api.com/json', timeout=3) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    ISP = f"{data.get('countryCode', '')}-{data.get('org', '')}".replace(' ', '_')
                    return
    except:
        pass

    ISP = 'Unknown'


async def get_ip():
    """Get public IP"""
    global CurrentDomain, Tls, CurrentPort
    if not DOMAIN or DOMAIN == 'your-domain.com':
        try:
            import aiohttp
            async with aiohttp.ClientSession() as session:
                async with session.get('https://api-ipv4.ip.sb/ip', timeout=5) as resp:
                    if resp.status == 200:
                        ip = await resp.text()
                        CurrentDomain = ip.strip()
                        Tls = 'none'
                        CurrentPort = PORT
        except:
            CurrentDomain = 'change-your-domain.com'
            Tls = 'tls'
            CurrentPort = 443
    else:
        CurrentDomain = DOMAIN
        Tls = 'tls'
        CurrentPort = 443


# ============== Protocol Handlers ==============
class ProxyHandler:
    def __init__(self, uuid: str):
        self.uuid = uuid
        self.uuid_bytes = bytes.fromhex(uuid)

    async def handle_vless(self, websocket, first_msg: bytes) -> bool:
        """Handle VLESS protocol"""
        try:
            if len(first_msg) < 18 or first_msg[0] != 0:
                return False

            # Verify UUID
            if first_msg[1:17] != self.uuid_bytes:
                return False

            i = first_msg[17] + 19
            if i + 3 > len(first_msg):
                return False

            port = struct.unpack('!H', first_msg[i:i+2])[0]
            i += 2
            atyp = first_msg[i]
            i += 1

            # Parse address
            host = ''
            if atyp == 1:  # IPv4
                if i + 4 > len(first_msg):
                    return False
                host = '.'.join(str(b) for b in first_msg[i:i+4])
                i += 4
            elif atyp == 2:  # Domain
                if i >= len(first_msg):
                    return False
                host_len = first_msg[i]
                i += 1
                if i + host_len > len(first_msg):
                    return False
                host = first_msg[i:i+host_len].decode()
                i += host_len
            elif atyp == 3:  # IPv6
                if i + 16 > len(first_msg):
                    return False
                host = ':'.join(f'{(first_msg[j] << 8) + first_msg[j+1]:04x}'
                              for j in range(i, i+16, 2))
                i += 16
            else:
                return False

            if is_blocked_domain(host):
                await websocket.close()
                return False

            # Send response
            await websocket.send_bytes(bytes([0, 0]))

            # Connect to target
            resolved_host = await resolve_host(host)
            reader, writer = await asyncio.open_connection(resolved_host, port)

            # Forward remaining data
            if i < len(first_msg):
                writer.write(first_msg[i:])
                await writer.drain()

            # Bidirectional forwarding
            async def forward_ws_to_tcp():
                try:
                    async for msg in websocket:
                        if msg.type == web.WSMsgType.BINARY:
                            writer.write(msg.data)
                            await writer.drain()
                except:
                    pass
                finally:
                    writer.close()
                    await writer.wait_closed()

            async def forward_tcp_to_ws():
                try:
                    while True:
                        data = await reader.read(4096)
                        if not data:
                            break
                        await websocket.send_bytes(data)
                except:
                    pass

            await asyncio.gather(forward_ws_to_tcp(), forward_tcp_to_ws())
            return True

        except Exception as e:
            if DEBUG:
                logger.error(f"VLESS handler error: {e}")
            return False

    async def handle_trojan(self, websocket, first_msg: bytes) -> bool:
        """Handle Trojan protocol"""
        try:
            if len(first_msg) < 58:
                return False

            received_hash_bytes = first_msg[:56]

            # Verify password - support standard UUID and UUID without dashes
            hash_obj1 = hashlib.sha224()
            hash_obj1.update(self.uuid.encode())
            expected_hash_hex1 = hash_obj1.hexdigest()

            standard_uuid = UUID
            hash_obj2 = hashlib.sha224()
            hash_obj2.update(standard_uuid.encode())
            expected_hash_hex2 = hash_obj2.hexdigest()

            received_hash_hex = received_hash_bytes.decode('ascii', errors='ignore')

            if received_hash_hex != expected_hash_hex1 and received_hash_hex != expected_hash_hex2:
                return False

            offset = 56
            if first_msg[offset:offset+2] == b'\r\n':
                offset += 2

            cmd = first_msg[offset]
            if cmd != 1:
                return False
            offset += 1

            atyp = first_msg[offset]
            offset += 1

            # Parse address
            host = ''
            if atyp == 1:  # IPv4
                host = '.'.join(str(b) for b in first_msg[offset:offset+4])
                offset += 4
            elif atyp == 3:  # Domain
                host_len = first_msg[offset]
                offset += 1
                host = first_msg[offset:offset+host_len].decode()
                offset += host_len
            elif atyp == 4:  # IPv6
                host = ':'.join(f'{(first_msg[j] << 8) + first_msg[j+1]:04x}'
                              for j in range(offset, offset+16, 2))
                offset += 16
            else:
                return False

            port = struct.unpack('!H', first_msg[offset:offset+2])[0]
            offset += 2

            if first_msg[offset:offset+2] == b'\r\n':
                offset += 2

            if is_blocked_domain(host):
                await websocket.close()
                return False

            # Connect to target
            resolved_host = await resolve_host(host)
            reader, writer = await asyncio.open_connection(resolved_host, port)

            if offset < len(first_msg):
                writer.write(first_msg[offset:])
                await writer.drain()

            async def forward_ws_to_tcp():
                try:
                    async for msg in websocket:
                        if msg.type == web.WSMsgType.BINARY:
                            writer.write(msg.data)
                            await writer.drain()
                except:
                    pass
                finally:
                    writer.close()
                    await writer.wait_closed()

            async def forward_tcp_to_ws():
                try:
                    while True:
                        data = await reader.read(4096)
                        if not data:
                            break
                        await websocket.send_bytes(data)
                except:
                    pass

            await asyncio.gather(forward_ws_to_tcp(), forward_tcp_to_ws())
            return True

        except Exception as e:
            if DEBUG:
                logger.error(f"Trojan handler error: {e}")
            return False

    async def handle_shadowsocks(self, websocket, first_msg: bytes) -> bool:
        """Handle Shadowsocks protocol"""
        try:
            if len(first_msg) < 7:
                return False

            offset = 0
            atyp = first_msg[offset]
            offset += 1

            # Parse address
            host = ''
            if atyp == 1:  # IPv4
                if offset + 4 > len(first_msg):
                    return False
                host = '.'.join(str(b) for b in first_msg[offset:offset+4])
                offset += 4
            elif atyp == 3:  # Domain
                if offset >= len(first_msg):
                    return False
                host_len = first_msg[offset]
                offset += 1
                if offset + host_len > len(first_msg):
                    return False
                host = first_msg[offset:offset+host_len].decode()
                offset += host_len
            elif atyp == 4:  # IPv6
                if offset + 16 > len(first_msg):
                    return False
                host = ':'.join(f'{(first_msg[j] << 8) + first_msg[j+1]:04x}'
                              for j in range(offset, offset+16, 2))
                offset += 16
            else:
                return False

            if offset + 2 > len(first_msg):
                return False
            port = struct.unpack('!H', first_msg[offset:offset+2])[0]
            offset += 2

            if is_blocked_domain(host):
                await websocket.close()
                return False

            # Connect to target
            resolved_host = await resolve_host(host)
            reader, writer = await asyncio.open_connection(resolved_host, port)

            if offset < len(first_msg):
                writer.write(first_msg[offset:])
                await writer.drain()

            async def forward_ws_to_tcp():
                try:
                    async for msg in websocket:
                        if msg.type == web.WSMsgType.BINARY:
                            writer.write(msg.data)
                            await writer.drain()
                except:
                    pass
                finally:
                    writer.close()
                    await writer.wait_closed()

            async def forward_tcp_to_ws():
                try:
                    while True:
                        data = await reader.read(4096)
                        if not data:
                            break
                        await websocket.send_bytes(data)
                except:
                    pass

            await asyncio.gather(forward_ws_to_tcp(), forward_tcp_to_ws())
            return True

        except Exception as e:
            if DEBUG:
                logger.error(f"Shadowsocks handler error: {e}")
            return False


# ============== HTTP Handlers ==============
async def websocket_handler(request):
    """WebSocket handler"""
    ws = web.WebSocketResponse()
    await ws.prepare(request)

    CUUID = UUID.replace('-', '')
    path = request.path

    if f'/{WS_PATH}' not in path:
        await ws.close()
        return ws

    proxy = ProxyHandler(CUUID)

    try:
        first_msg = await asyncio.wait_for(ws.receive(), timeout=5)
        if first_msg.type != web.WSMsgType.BINARY:
            await ws.close()
            return ws

        msg_data = first_msg.data

        # Try VLESS
        if len(msg_data) > 17 and msg_data[0] == 0:
            if await proxy.handle_vless(ws, msg_data):
                return ws

        # Try Trojan
        if len(msg_data) >= 56:
            if await proxy.handle_trojan(ws, msg_data):
                return ws

        # Try Shadowsocks
        if len(msg_data) > 0 and msg_data[0] in (1, 3, 4):
            if await proxy.handle_shadowsocks(ws, msg_data):
                return ws

        await ws.close()

    except asyncio.TimeoutError:
        await ws.close()
    except Exception as e:
        if DEBUG:
            logger.error(f"WebSocket handler error: {e}")
        await ws.close()

    return ws


async def http_handler(request):
    """HTTP handler"""
    if request.path == '/':
        try:
            with open('index.html', 'r', encoding='utf-8') as f:
                content = f.read()
            return web.Response(text=content, content_type='text/html')
        except:
            return web.Response(text='Hello World!', content_type='text/html')

    elif request.path == '/health':
        return web.Response(text='OK', content_type='text/plain')

    elif request.path == f'/{SUB_PATH}':
        await get_isp()
        await get_ip()

        name_part = f"{NAME}-{ISP}" if NAME else ISP
        tls_param = 'tls' if Tls == 'tls' else 'none'
        ss_tls_param = 'tls;' if Tls == 'tls' else ''

        # Generate subscription links
        vless_url = f"vless://{UUID}@{CurrentDomain}:{CurrentPort}?encryption=none&security={tls_param}&sni={CurrentDomain}&fp=chrome&type=ws&host={CurrentDomain}&path=%2F{WS_PATH}#{name_part}"
        trojan_url = f"trojan://{UUID}@{CurrentDomain}:{CurrentPort}?security={tls_param}&sni={CurrentDomain}&fp=chrome&type=ws&host={CurrentDomain}&path=%2F{WS_PATH}#{name_part}"

        ss_method_password = base64.b64encode(f"none:{UUID}".encode()).decode()
        ss_url = f"ss://{ss_method_password}@{CurrentDomain}:{CurrentPort}?plugin=v2ray-plugin;mode%3Dwebsocket;host%3D{CurrentDomain};path%3D%2F{WS_PATH};{ss_tls_param}sni%3D{CurrentDomain};skip-cert-verify%3Dtrue;mux%3D0#{name_part}"

        subscription = f"{vless_url}\n{trojan_url}\n{ss_url}"
        base64_content = base64.b64encode(subscription.encode()).decode()

        return web.Response(text=base64_content + '\n', content_type='text/plain')

    return web.Response(status=404, text='Not Found')


# ============== Main ==============
async def add_access_task():
    """Add auto access task"""
    if not AUTO_ACCESS or not DOMAIN:
        return

    full_url = f"https://{DOMAIN}/{SUB_PATH}"
    try:
        import aiohttp
        async with aiohttp.ClientSession() as session:
            await session.post("https://oooo.serv00.net/add-url",
                             json={"url": full_url},
                             headers={'Content-Type': 'application/json'})
        logger.info('Auto access task added')
    except:
        pass


async def main():
    app = web.Application()

    # Routes
    app.router.add_get('/', http_handler)
    app.router.add_get('/health', http_handler)
    app.router.add_get(f'/{SUB_PATH}', http_handler)
    app.router.add_get(f'/{WS_PATH}', websocket_handler)

    # Start server
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, '0.0.0.0', PORT)
    await site.start()

    logger.info(f"Server running on port {PORT}")
    logger.info(f"WebSocket path: /{WS_PATH}")
    logger.info(f"Subscription path: /{SUB_PATH}")

    await add_access_task()

    try:
        await asyncio.Future()
    except KeyboardInterrupt:
        pass
    finally:
        await runner.cleanup()


if __name__ == '__main__':
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nServer stopped")
