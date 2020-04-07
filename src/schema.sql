CREATE TABLE users (email_key varchar(255), apikey text,  PRIMARY KEY(apikey));
CREATE TABLE lectures (id int, label varchar(255), PRIMARY KEY (id));
CREATE TABLE questions (lec int, q int, question text, PRIMARY KEY (lec, q));

lec_qcount: SELECT questions.lec, COUNT(questions.q) AS qcount FROM questions GROUP BY questions.lec;
QUERY leclist: SELECT lectures.id, lectures.label, lec_qcount.qcount FROM lectures LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec);
QUERY lecture: SELECT * FROM lectures WHERE id = ?;
QUERY qs_by_lec: SELECT * FROM questions WHERE lec = ?;
QUERY users_by_apikey: SELECT * FROM users WHERE apikey = ?;
QUERY all_users: SELECT email_key FROM users;


