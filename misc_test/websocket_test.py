import time
import websocket
import json
import argparse

parser = argparse.ArgumentParser()

parser.add_argument('--ip', type=str, required=True)
parser.add_argument('--id', type=str, required=True)

args = parser.parse_args()

ws = websocket.WebSocket()

ws.connect(f"ws://{args.ip}:4000/orders/ws")

jsonreq = {
            'OrderType': "Sell",
            'Amount': 3,
            'Price': 3,
            'Symbol': "AAPL",
            'TraderId': f"{args.id}",            
        }

ws.send(json.dumps(jsonreq))
# ws.send("Hello")
# time.sleep(20)
while True:
    print(ws.recv())
