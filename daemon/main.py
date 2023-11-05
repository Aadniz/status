#!/usr/bin/python3

"""
Example daemon script to communicate with the rust application
"""

import zmq
import sys


PROTOCOL = "tcp"
HOST = "127.0.0.1"
PORT = 5747



args = sys.argv

if 1 >= len(args):
    argz = "help"
else:
    argz = " ".join(args[1:])


# Prepare our context and sockets
context = zmq.Context()

# Socket to talk to server
socket = context.socket(zmq.DEALER)
socket.connect(f"{PROTOCOL}://{HOST}:{PORT}")

# Send request
socket.send(argz.encode("UTF-8"))

# Get reply
message = socket.recv_multipart()

# Decode each byte string and join them together
message_str = "".join(part.decode('utf-8') for part in message)
print(message_str)