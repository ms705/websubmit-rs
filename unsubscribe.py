import requests

def remove_user_data(session):
  session.get('http://localhost:8000/leclist')
  session.post('http://localhost:8000/apikey/remove_data')

if __name__ == '__main__':
  session = requests.Session()
  users = set()
  with open("un.txt", 'r') as f:
    lines = f.readlines()
    for line in lines:
      #login
      login = 'http://localhost:8000/login'
      session.get(login)
      apikey_check = 'http://localhost:8000/apikey/check'
      login_hash = {'key' : line.rstrip()}
      session.post(apikey_check, login_hash)
      remove_user_data(session)
