SET echo;
CREATE TABLE users (PII_email varchar(255), apikey varchar(255), is_admin int, PRIMARY KEY (apikey));
CREATE TABLE lectures (id int, label varchar(255), PRIMARY KEY (id));
CREATE TABLE questions (id text, lec int, q int, question text,  PRIMARY KEY (id));
CREATE TABLE answers (id text, email varchar(255), lec int, q int, answer text, submitted_at datetime, FOREIGN KEY (email) REFERENCES users(PII_email), PRIMARY KEY (id));

CREATE VIEW lec_qcount as '"SELECT questions.lec, COUNT(questions.q) AS qcount FROM questions GROUP BY questions.lec"';
