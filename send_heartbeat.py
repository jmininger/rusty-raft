from socket import socket, AF_INET, SOCK_STREAM
import json
import struct
import time


# def send_heartbeat(node_addr):
#     # Send a heartbeat to the orchestrator
#     requests.post("http://orchestrator:5000/heartbeat")


def dial_peer(peer_addr):
    sock = socket(AF_INET, SOCK_STREAM)
    sock.connect(peer_addr)
    exchange_identity(sock)
    n = 0
    while True:
        n += 1
        msg = {"peer_name": "python_client", "rpc": {"id": n, "method": "ping", "params": []}}
        send_length_prefixed_message(sock, msg)
        data = sock.recv(1024)
        if not data:
            break
        print("Received data from node", data)
        time.sleep(1)

    # dial_addr = sock.getsockname()
    # identity = "common_name": "foobar", "dial_addr": dial_addr}


def exchange_identity(sock):
    common_name = b"python_client\r\n"
    sock.sendall(common_name)
    (addr, port) = sock.getsockname()
    msg = addr + ":" + str(port) + "\r\n"
    sock.sendall(msg.encode())
    chunk1 = sock.recv(1024)
    print(chunk1)
    chunk2 = sock.recv(1024)
    print(chunk2)


def send_length_prefixed_message(sock, message):
    # Serialize the JSON message to bytes
    json_bytes = json.dumps(message).encode('utf-8')
    # Pack the length of the message as a 4-byte big-endian integer
    length_prefix = struct.pack('>I', len(json_bytes))
    print("Sending message of length", len(json_bytes))
    print("Message:", length_prefix + json_bytes)
    # Send the length prefix and the message
    sock.sendall(length_prefix + json_bytes)
