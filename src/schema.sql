CREATE TABLE users (email varchar(255), apikey text, is_admin tinyint, PRIMARY KEY (apikey));
CREATE TABLE lectures (id int, label varchar(255), PRIMARY KEY (id));
CREATE TABLE questions (lec int, q int, question text, PRIMARY KEY (lec, q));
CREATE TABLE answers (user varchar(255), lec int, q int, answer text, PRIMARY KEY (user, lec, q));

lec_qcount: SELECT lectures.id, COUNT(lectures.label) FROM lectures JOIN questions ON (lectures.id = questions.lec) GROUP BY lectures.id;
QUERY leclist: SELECT lectures.* FROM lectures LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.id);
QUERY lecture: SELECT * FROM lectures WHERE id = ?;
QUERY qs_by_lec: SELECT * FROM questions WHERE lec = ?;
QUERY answers_by_lec: SELECT * FROM answers WHERE lec = ?;
QUERY users_by_apikey: SELECT * FROM users WHERE apikey = ?;
