import time
from flask import Flask
from waitress import serve

app = Flask(__name__)

@app.route('/')
def hello_world():
    return 'Hello python!\n'

if __name__ == '__main__':
    print("finish loading at: " + str(int(time.time() * 1000)) + "\n")
    serve(app, host='0.0.0.0',port=int(8079))
