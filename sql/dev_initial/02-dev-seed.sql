-- root user (at id = 0 for sys)
INSERT INTO "user" 
    (id,  typ, username, cid, ctime, mid, mtime) VALUES 
    (0, 'Sys', 'root',  0,   now(), 0,   now());

-- root org
INSERT INTO "org" 
    (id,  name, cid, ctime, mid, mtime) VALUES 
    (100, 'demo1 org',  0,   now(), 0,   now());

-- User demo1
INSERT INTO "user" 
    (id,  username, cid, ctime, mid, mtime) VALUES 
    (100, 'demo1 user',  0,   now(), 0,   now());

-- Agent mock-01 (with 'parrot' model) (id: 100)
INSERT INTO "agent"    
    (id,  org_id, owner_id, name,      cid, ctime, mid, mtime) VALUES
    (100, 100,    0,        'demo1 agent', 0,   now(), 0,   now());

