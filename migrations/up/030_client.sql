CREATE TABLE Clients (
    id SERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    status_id INTEGER NOT NULL,
    kind_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    FOREIGN KEY (status_id)
        REFERENCES Statuses(id),
    FOREIGN KEY (kind_id)
        REFERENCES Kinds(id)
)