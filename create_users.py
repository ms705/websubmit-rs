import requests
from faker import Faker
import hashlib

admin_key ='0ab1f0f6afc524bb5a36641978aa7d37e017d63e6387cef3324bb57d48154c39'
responses = 1
users_by_apikey = {}
NUM_USERS = 10
UNSUB = 5

intervals = []

# generates hash for a given email, using a secret word
def generate_hash(email):
  m = hashlib.sha256()
  m.update(email.encode('utf-8'))
  m.update(b'SECRET')
  return m.hexdigest()

def generate_user(session, email, apikey):
# register the user
  apikey_generate = 'http://localhost:8000/apikey/generate'
  email_data = {'email' : email}
  session.post(apikey_generate, data=email_data)

  # login
  apikey_check = 'http://localhost:8000/apikey/check'
  login_hash = {'key' : apikey}
  session.post(apikey_check, login_hash)

def add_lecture_and_question(session, lec_id):
  # adding a lecture
  lecture = {'lec_id' : lec_id, 'lec_label' : faker.word()}
  lec_add = 'http://localhost:8000/admin/lec/add'
  session.post(lec_add, data=lecture)

  # add 1 question
  lec_addr = f'http://localhost:8000/admin/lec/{lec_id}'
  session.get(lec_addr)
  for q in range(responses):
    q = {"q_id": str(q), "q_prompt": faker.sentence()}
    session.post(lec_addr, q)

  # return to the leclist
  session.get('http://localhost:8000/leclist')

def create_answer(session, lec_num, q_num):
  global stack
  question_url = f'http://localhost:8000/questions/{lec_num}'
  session.get(question_url)
  data = {}
  sentence = faker.sentence()
  data.update({f'q_{str(q_num)}': sentence})

  session.post(question_url, data=data)

if __name__ == '__main__':
  users_session = requests.Session()
  admin_session = requests.Session()
  faker = Faker()

  generate_user(admin_session, 'ekiziv@brown.edu', admin_key)
  add_lecture_and_question(admin_session, "0")

  for i in range (1):
    print("Creating user number:", i)
    response = users_session.get('http://localhost:8000/login')
    email = faker.email()
    email_key = email.split('@', 1)[0]
    apikey = generate_hash(email)
    generate_user(users_session, email, apikey)
    users_by_apikey[apikey] = email_key


  i = 0;
  # f_info = open("info.txt", 'w')
  f_un = open("un.txt", 'w')
  for api, email_key in users_by_apikey.items():
    if i >= (NUM_USERS - UNSUB):
      f_un.write(f'{api}\n')
    # f_info.write(f'{email_key}\n')
    i += 1





