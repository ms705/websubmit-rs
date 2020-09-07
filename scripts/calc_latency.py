from datetime import datetime
import re

if __name__ == '__main__':
  start_dict = {}
  end_dict = {}

  default = datetime.now()
  with open("end_times.txt", "r") as f:
    for line in f:
      if "id=\"answer\"" in line:
        answer = re.findall('^<td id="answer">(.+)<\/td>', line)[0]
      elif "*" in line:
        time = re.findall('<\/tr>\*(.+)', line)[0]
        new_time = time.strip('\n')
        datetime;
        if new_time == "0":
          print("time is zero for answer", answer)
          datetime = default
        else:
          datetime = datetime.strptime(new_time, '%Y-%m-%d %H:%M:%S.%f')
        end_dict[answer] = datetime

  with open("start_times.txt","r") as f:
    for line in f:
      key, val = line.split("*")
      new_time = val.strip('\n')
      datetime;
      if new_time == "0":
        datetime = default
      else:
        datetime = datetime.strptime(new_time, '%Y-%m-%d %H:%M:%S.%f')
      start_dict[key] = datetime

  with open("results.txt", "w") as f:
    for answer, start_time in start_dict.items():
      if answer in end_dict:
        # calculate latency and write it into a file
        end_time = end_dict[answer]
        delta = (end_time-start_time).total_seconds()*1000
        if delta == 0:
          print("nice!")
        f.write(f'{delta}\n')
      else:
        print("could not find this answer in end:", answer)

