from flask import Flask

app = Flask(__name__)

@app.route('/')
def hello_world():
    return 'Hello python!\n'

if __name__ == '__main__':
    from waitress import serve
    serve(app, host='0.0.0.0',port=int(8079))
