CREATE TABLE users (
   email    VARCHAR(255),
   apikey   VARCHAR(255),
   is_admin TINYINT,
   PRIMARY KEY (email)
);

CREATE TABLE lectures (
   id    INT,
   label VARCHAR(255),
   PRIMARY KEY (id)
);

CREATE TABLE questions (
   lec         INT,
   question_id INT,
   question    TEXT,
   PRIMARY KEY (lec, question_id),
   FOREIGN KEY (lec) REFERENCES lectures(id)
);

CREATE TABLE answers (
   email        VARCHAR(255),
   lec          INT,
   question_id  INT,
   answer       TEXT,
   submitted_at DATETIME,
   PRIMARY KEY (email, lec, question_id),
   FOREIGN KEY (lec) REFERENCES lectures(id),
   FOREIGN KEY (lec, question_id) REFERENCES questions(lec, question_id)
   FOREIGN KEY (email) REFERENCES users(email),
);

CREATE TABLE presenters (
   lec   INT,
   email VARCHAR(255),
   FOREIGN KEY (lec) REFERENCES lectures(id),
   FOREIGN KEY (email) REFERENCES users(email)
);

CREATE VIEW lec_qcount AS SELECT questions.lec, COUNT(questions.question_id) AS qcount FROM questions GROUP BY questions.lec;
