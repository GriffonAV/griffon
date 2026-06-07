# How to use :

1. Use this command at the root of the project
```
docker-compose up -d 
```
2. Go to 
```
http://localhost:3000
```

3. When asked for login enter :  
- user : `admin`  
- password : `admin`  

4. Once logged in press the dashboard icon on the top left corner of the page.  

5. use the folder interface to visit the dashboard you want, if the dashboards show errors, the sqlite plugin probably isn't loaded yet. Wait two minutes and retry. 

# How to enter data :





## Insert data directly in the sqlite database


### Prerequisite :  

- have sqlite installed

Depending on your distro the installation process might vary:  
- arch  
`yay sqlite`   
- ubuntu  
`sudo apt-get install sqlite`

### Steps

1. Use this command at the root of the project
```
sqlite3 grafana/datasources/reporting.sqlite
```

2. create a new table
```
CREATE TABLE system_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin TEXT NOT NULL,
    ram_usage REAL NOT NULL,
    cpu_usage REAL NOT NULL,
    disk_usage REAL NOT NULL,
    is_active INTEGER,
    version REAL NOT NULL
);
```

3. insert data into the table
```
INSERT INTO system_metrics (plugin, ram_usage, cpu_usage, disk_usage, is_active, version) VALUES
-- Initial versions
('Cleaner', 12, 15.5, 20, 1, 0.1),
('Scanner', 32, 18.2, 25, 1, 0.1),

-- Version 0.2
('Cleaner', 11, 14.8, 19, 1, 0.2),
('Scanner', 34, 19.5, 27, 1, 0.2),

-- Version 0.3
('Cleaner', 10, 13.6, 18, 1, 0.3),
('Scanner', 37, 21.0, 30, 1, 0.3),

-- Version 0.4
('Cleaner', 9, 12.9, 17, 1, 0.4),
('Scanner', 40, 22.7, 33, 1, 0.4),

-- Version 0.5
('Cleaner', 8, 11.7, 15, 1, 0.5),
('Scanner', 44, 24.8, 36, 1, 0.5),

-- Version 1.0
('Cleaner', 7, 10.5, 14, 1, 1.0),
('Scanner', 48, 27.2, 40, 1, 1.0);
```

4. exit the program
```
.quit
```

# Turn off the instance

To turn off the instance run this command:

```
sudo docker compose down
```

if you want to remove the containers add the `-v` flag to the above command