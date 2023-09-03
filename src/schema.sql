-- DATA SUBJECT TABLE.
CREATE DATA_SUBJECT TABLE users (
    email varchar(255),
    apikey varchar(255),
    is_admin int,
    PRIMARY KEY (email),
    UNIQUE (apikey)
);

-- lectures and questions table are unsharded.
CREATE TABLE lectures (
    id int,
    label varchar(255),
    PRIMARY KEY (id)
);
-- question_number: number *within* lecture
CREATE TABLE questions (
    id int AUTO_INCREMENT,
    lecture_id int,
    question_number int,
    question text,
    PRIMARY KEY (id),
    FOREIGN KEY (lecture_id) REFERENCES lectures(id)
);

-- Answers are owned by the student that provided the answer.
-- id = format!('{}-{}', email, question_id)
CREATE TABLE answers (
    id varchar(255),
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

CREATE VIEW lectures_with_question_counts AS '"
(
    SELECT lectures.id AS id, lectures.label, 0 AS U_c
    FROM lectures LEFT JOIN questions ON (lectures.id = questions.lecture_id)
    WHERE questions.id IS NULL
    GROUP BY lectures.id, lectures.label
)
UNION
(
    SELECT lectures.id AS id, lectures.label, COUNT(*) AS U_c
    FROM lectures JOIN questions ON (lectures.id = questions.lecture_id)
    GROUP BY lectures.id, lectures.label
)
ORDER BY id
"';