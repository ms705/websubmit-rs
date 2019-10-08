CREATE TABLE users (email varchar(255), apikey text, is_admin tinyint, PRIMARY KEY (apikey));
CREATE TABLE lectures (id int, label varchar(255), PRIMARY KEY (id));
CREATE TABLE questions (lec int, q int, question text, PRIMARY KEY (lec, q));
CREATE TABLE answers (`user` varchar(255), lec int, q int, answer text, submitted_at datetime, PRIMARY KEY (user, lec, q));

submitted: SELECT answers.`user`, answers.lec, COUNT(answers.q) AS num_answered FROM answers GROUP BY answers.`user`, answers.lec;
lec_qcount: SELECT questions.lec, COUNT(questions.q) AS qcount FROM questions GROUP BY questions.lec;
-- QUERY leclist: SELECT lectures.id, lectures.label, submitted.`user`, lec_qcount.qcount, submitted.num_answered FROM lectures LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec) LEFT JOIN submitted ON (lectures.id = submitted.lec) WHERE submitted.`user` = ?;
QUERY leclist: SELECT lectures.id, lectures.label, lec_qcount.qcount FROM lectures LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec);
QUERY lecture: SELECT * FROM lectures WHERE id = ?;
QUERY qs_by_lec: SELECT * FROM questions WHERE lec = ?;
QUERY answers_by_lec: SELECT * FROM answers WHERE lec = ?;
QUERY users_by_apikey: SELECT * FROM users WHERE apikey = ?;
