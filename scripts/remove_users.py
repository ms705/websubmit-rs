import urllib.request
import webbrowser
import os
import requests
import hashlib
from faker import Faker
import random
from time import perf_counter_ns
import time
import threading
from datetime import datetime
from bs4 import BeautifulSoup
import queue


admin_key ='0ab1f0f6afc524bb5a36641978aa7d37e017d63e6387cef3324bb57d48154c39'
responses = 100 #might need to tune this!
users = list()
start_times = {}
end_times = {}
NUM_USERS = 60
users_by_apikey = {}
submitted = {}

stack = queue.Queue() # should be sync

starting_time = datetime.now()
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
  if apikey is not admin_key:
    users.append(apikey)

def create_answer(session, lec_num, q_num, email_key):
  global stack
  question_url = f'http://localhost:8000/questions/{lec_num}'
  session.get(question_url)
  data = {}
  sentence = faker.sentence()
  data.update({f'q_{str(q_num)}': sentence})

  passed = (datetime.now()-starting_time).total_seconds()*1000
  intervals.append(passed)
  val = (email_key, q_num)
  stack.put(val)
  start_times[sentence] = datetime.now()

  session.post(question_url, data=data)

def add_lecture_and_question(session, lec_id):
  # adding a lecture
  lecture = {'lec_id' : lec_id, 'lec_label' : faker.word()}
  lec_add = 'http://localhost:8000/admin/lec/add'
  session.post(lec_add, data=lecture)

  # add 1 question
  lec_addr = f'http://localhost:8000/admin/lec/{lec_id}'
  session.get(lec_addr)
  data = {}
  for q in range(responses):
    q1 = {"q_id": str(q), "q_prompt": faker.sentence()}
    data.update(q1)
  session.post(lec_addr, data)

  # return to the leclist
  session.get('http://localhost:8000/leclist')

def remove_user_data(session):
  session.get('http://localhost:8000/leclist')
  session.post('http://localhost:8000/apikey/remove_data')

def constant_load(session):
  global users_by_apikey
  num_users_processed = 0
  while True:
    for user in users[:(NUM_USERS-10)]:
      #logging
      login = 'http://localhost:8000/apikey/check'
      login_hash = {'key' : user}
      session.post(login, login_hash)
      num_users_processed += 1
      print(num_users_processed)
      email_key = users_by_apikey[user]
      for q_num in range(responses):
        create_answer(session, "0", q_num, email_key)

def start_polling(session):
  global stack
  while True:
    if not stack.empty():
      (user, q_num) = stack.get()
      found = False
      while not found:
        response = session.get(f'http://localhost:8000/admin/answers/{user}/{q_num}')
        soup = BeautifulSoup(response.content, 'html.parser')
        answers = soup.find_all('tr')
        if len(answers) > 0:
          end_times[answers[-1]] = datetime.now()
          found = True

def unregister(session):
  global start_times
  time.sleep(20)
  start_times["START"] = 0

  start_string = f'<td id=\"answer\">START</td>\n</tr>'
  end_times[start_string] = 0

  to_unregister = users[(NUM_USERS-10):]
  for user in to_unregister:
    login = 'http://localhost:8000/apikey/check'
    login_hash = {'key' : user}
    session.post(login, login_hash)
    remove_user_data(session)

  print("unsubscribed")
  start_times["STOP"] = 0
  end_string = f'<td id=\"answer\">STOP</td>\n</tr>'
  end_times[end_string] = 0
  time.sleep(20)

  with open('start_times.txt', 'w') as f:
    for k, v in start_times.items():
      f.write(f'{k}*{v}\n')
  with open('end_times.txt', 'w') as f2:
    for k, v in end_times.items():
      f2.write(f'{k}*{v}\n')
  with open('intervals.txt', 'w') as f3:
    for v in intervals:
      f3.write(f'{v}\n')
  print("done copying")

if __name__ == '__main__':
  users_session = requests.Session()
  admin_session = requests.Session()
  unregister_session = requests.Session()

  faker = Faker()

  # admin thread
  generate_user(admin_session, 'ekiziv@brown.edu', admin_key)
  add_lecture_and_question(admin_session, "0")
  x = threading.Thread(target=start_polling, args=(admin_session, ))

  # users thread
  for i in range (NUM_USERS):
    print("Creating user number:", i)
    response = users_session.get('http://localhost:8000/login')

    email = faker.email()
    email_key = email.split('@', 1)[0]
    apikey = generate_hash(email)
    generate_user(users_session, email, apikey)
    users_by_apikey[apikey] = email_key

  i = 0;
  f_info = open("info.txt", 'w')
  f_un = open("un.txt", 'w')
  for api, email_key in users_by_apikey.items():
    if i < NUM_USERS-20:
      f_info.write(f'{email_key}\n')
    else:
      f_un.write(f'{api}\n')
    i += 1
  print(users_by_apikey)

  y = threading.Thread(target=constant_load, args=(users_session, ))
  z = threading.Thread(target=unregister, args=(unregister_session,))

  # x.start()
  # y.start()
  # z.start()




