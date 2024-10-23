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
    print("Connected to peer", peer_addr)
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
    [name, addr] = read_lines_from_socket(sock, 2)
    print("Received identity from peer", name, addr)


def send_length_prefixed_message(sock, message):
    # Serialize the JSON message to bytes
    json_bytes = json.dumps(message).encode('utf-8')
    # Pack the length of the message as a 4-byte big-endian integer
    length_prefix = struct.pack('>I', len(json_bytes))
    print("Sending message of length", len(json_bytes))
    print("Message:", length_prefix + json_bytes)
    # Send the length prefix and the message
    sock.sendall(length_prefix + json_bytes)


def read_lines_from_socket(sock, n):
    received_data = ""
    line_breaks = 0
    lines = []
    while line_breaks < n:
        data = sock.recv(1024).decode('utf-8')  # Read a chunk of data from the socket
        if not data:
            break  # Connection closed by the client
        received_data += data
        partial_lines = received_data.split('\n')
        lines.extend(partial_lines[:-1])
        line_breaks += len(partial_lines) - 1
        received_data = partial_lines[-1]
        print("Received ", len(partial_lines) - 1, "lines")
    if received_data:
        lines.append(received_data)
    return lines
