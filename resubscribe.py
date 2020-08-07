import requests

if __name__ == '__main__':
  session = requests.Session()
  users = set()
  with open("imported_data.txt", 'r') as f:
    lines = f.readlines()
    for line in lines:
      sanitized = line.rstrip()
      info = sanitized.split('*')
      #login
      login = 'http://localhost:8000/login'
      session.get(login)
      resubscribe = 'http://localhost:8000/apikey/resubscribe'
      login_hash = {'key' : info[0], 'data': info[1]}
      session.post(resubscribe, login_hash)
