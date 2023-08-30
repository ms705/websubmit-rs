-- DATA SUBJECT TABLE.
CREATE DATA_SUBJECT TABLE users (
    email varchar(255),
    apikey varchar(255),
    is_admin int,
    PRIMARY KEY (email)
);

-- lectures and questions table are unsharded.
CREATE TABLE lectures (
    id int AUTO_INCREMENT,
    label varchar(255),
    PRIMARY KEY (id)
);
CREATE TABLE questions (
    id int AUTO_INCREMENT,
    lecture_id int,
    question text,
    PRIMARY KEY (id),
    FOREIGN KEY (lecture_id) REFERENCES lectures(id)
);

-- Answers are owned by the student that provided the answer.
CREATE TABLE answers (
    id int AUTO_INCREMENT,
    email varchar(255),
    question_id int,
    answer text,
    submitted_at datetime,
    PRIMARY KEY (id),
    FOREIGN KEY (email) OWNED_BY users(email),
    FOREIGN KEY (question_id) REFERENCES questions(id)
);

-- A present owns the record that marks them as a presenter of some lecture.
CREATE TABLE presenters (
    id int AUTO_INCREMENT,
    lecture_id int,
    email varchar(255) OWNED_BY users(email),
    PRIMARY KEY (id),
    FOREIGN KEY (lecture_id) REFERENCES lectures(id)
);

-- View for tracking number of questions assigned to a lecture.
CREATE VIEW lec_qcount as '"SELECT questions.lecture_id, COUNT(questions.id) AS qcount FROM questions GROUP BY questions.lecture_id"';
