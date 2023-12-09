-- root user (at id = 0)
INSERT INTO "user" 
    (id, username, cid, ctime, mid, mtime) VALUES 
    (0, 'root',  0,   now(), 0,   now());

-- User demo1
INSERT INTO "user" 
    (username, cid, ctime, mid, mtime) VALUES 
    ('demo1',  0,   now(), 0,   now());