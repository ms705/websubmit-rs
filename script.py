import urllib.request
import webbrowser
import os
import requests
import hashlib
from faker import Faker

admin_key ='0ab1f0f6afc524bb5a36641978aa7d37e017d63e6387cef3324bb57d48154c39'

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
  apikey_generate = 'http://localhost:8000/apikey/generate'
  email_data = {'email' : email}
  session.post(apikey_generate, data=email_data)

  # login
  key = generate_hash(email)
  apikey_check = 'http://localhost:8000/apikey/check'
  login_hash = {'key' : key}
  session.post(apikey_check, login_hash)

def create_answer(session, q_num):
  question_url = f'http://localhost:8000/questions/{q_num}'
  session.get(question_url)
  # answer one question
  data = {f'q_{q_num}': faker.sentence()}
  session.post(question_url, data=data)


def add_lecture_and_question(session, lec_id, lec_label):
   # adding a lecture
  lecture = {'lec_id' : lec_id, 'lec_label' : lec_label}
  lec_add = 'http://localhost:8000/admin/lec/add'
  session.post(lec_add, data=lecture)

  # add one question
  lec_addr = f'http://localhost:8000/admin/lec/{lec_id}'
  session.get(lec_addr)
  q1 = {"q_id": "0", "q_prompt": faker.sentence()}
  session.post(lec_addr, q1)

  # return to the leclist
  session.get('http://localhost:8000/leclist')


# I need a notion of continously fetching the url that gets output
if __name__ == '__main__':
  session = requests.session()
  faker = Faker()

  generate_user(session, 'ekiziv@brown.edu')
  lec_id = "0"
  add_lecture_and_question(session, lec_id, faker.word())


  #generate 10 random users and each of them with an answer
  for i in range (5):
    response = session.get('http://localhost:8000/login')
    email = faker.email()
    generate_user(session, email)
    create_answer(session, lec_id)

  # login as admin and see the users
  login = 'http://localhost:8000/apikey/check'
  login_hash = {'key' : admin_key}
  session.post(login, login_hash)
  response = session.get(f'http://localhost:8000/answers/{lec_id}')

  # visualizing the final page.decode('utf-8')
  the_page = response.text
  file = write_html(the_page)
  webbrowser.get('chrome').open(file)
 # webbrowser.get('chrome').open_new_tab('http://localhost:6033/graph.html')


