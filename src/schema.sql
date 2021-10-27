CREATE TABLE users (email varchar(255), apikey text, is_admin tinyint, PRIMARY KEY (apikey));
CREATE TABLE lectures (id int, label varchar(255), PRIMARY KEY (id));
CREATE TABLE questions (lec int, q int, question text, PRIMARY KEY (lec, q));
CREATE TABLE answers (`user` varchar(255), lec int, q int, answer text, submitted_at datetime, PRIMARY KEY (user, lec, q));

CREATE VIEW lec_qcount as SELECT questions.lec, COUNT(questions.q) AS qcount FROM questions GROUP BY questions.lec;
QUERY leclist: SELECT lectures.id, lectures.label, lec_qcount.qcount FROM lectures LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec);
-- WHERE lectures.id = ?;
QUERY lecture: SELECT * FROM lectures WHERE id = ?;
QUERY qs_by_lec: SELECT * FROM questions WHERE lec = ?;
QUERY answers_by_lec: SELECT * FROM answers WHERE lec = ?;
QUERY users_by_apikey: SELECT * FROM users WHERE apikey = ?;
QUERY my_answers_for_lec: SELECT answers.* FROM answers WHERE answers.lec = ? AND answers.`user` = ?;
QUERY all_users: SELECT email, is_admin, apikey FROM users;
