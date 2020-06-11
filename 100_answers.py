import urllib.request
import webbrowser
import os
import requests
import hashlib
from faker import Faker
import random
from time import perf_counter_ns

admin_key ='0ab1f0f6afc524bb5a36641978aa7d37e017d63e6387cef3324bb57d48154c39'
responses = 100

def write_html(page):
  f = open('page.html','w')
  f.write(page)
  f.close()

  filename = 'file:///'+os.getcwd()+'/' + 'page.html'
  return filename

# generates hash for a given email, using a secret word
def generate_hash(email):
  m = hashlib.sha256()
  m.update(email.encode('utf-8'))
  m.update(b'SECRET')
  return m.hexdigest()

def generate_user(session, email):
# register the user
  apikey_generate = 'http://0.0.0.0:8000/apikey/generate'
  email_data = {'email' : email}
  session.post(apikey_generate, data=email_data)

  # login
  key = generate_hash(email)
  apikey_check = 'http://0.0.0.0:8000/apikey/check'
  login_hash = {'key' : key}
  session.post(apikey_check, login_hash)

def create_answer(session, lec_num):
  question_url = f'http://0.0.0.0:8000/questions/{lec_num}'
  session.get(question_url)
  data = {}
  start = perf_counter_ns()
  for q in range(responses):
    data.update({f'q_{str(q)}': faker.sentence()})
  end = perf_counter_ns()
  f=open("write_time.txt", "a+")
  f.write(f'{end-start}\n')
  session.post(question_url, data=data)


def add_lecture_and_question(session, lec_id):
  # adding a lecture
  lecture = {'lec_id' : lec_id, 'lec_label' : faker.word()}
  lec_add = 'http://0.0.0.0:8000/admin/lec/add'
  session.post(lec_add, data=lecture)

  # add 1 question
  lec_addr = f'http://0.0.0.0:8000/admin/lec/{lec_id}'
  session.get(lec_addr)
  data = {}
  for q in range(responses):
    q1 = {"q_id": str(q), "q_prompt": faker.sentence()}
    data.update(q1)
  session.post(lec_addr, data)

  # return to the leclist
  session.get('http://0.0.0.0:8000/leclist')

def lookup_current_users(session):
  login = 'http://0.0.0.0:8000/apikey/check'
  login_hash = {'key' : admin_key}
  session.post(login, login_hash)
  response = session.get(f'http://0.0.0.0:8000/admin/users')
  return response

def lookup_answers(session, lec_id):
  login = 'http://0.0.0.0:8000/apikey/check'
  login_hash = {'key' : admin_key}
  session.post(login, login_hash)
  response = session.get(f'http://0.0.0.0:8000/answers/{lec_id}')
  return response

def visualize_results(session, response):
  the_page = response.text
  file = write_html(the_page)
  webbrowser.get('chrome').open(file)
  webbrowser.get('chrome').open_new_tab('http://localhost:6033/graph.html')


if __name__ == '__main__':
  session = requests.session()
  faker = Faker()

  generate_user(session, 'ekiziv@brown.edu')
  add_lecture_and_question(session, "0")

  #generate 10 random users and each of them with an answer
  for i in range (100):
    print("Creating user number:", i)
    response = session.get('http://0.0.0.0:8000/login')
    email = faker.email()
    generate_user(session, email)
    create_answer(session, "0")

  res = lookup_current_users(session)
  visualize_results(session, res)



