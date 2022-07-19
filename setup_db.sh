#!/bin/sh

echo "DROP TABLE IF EXISTS Users;" | sqlite3 dev.db
echo "DROP TABLE IF EXISTS Users;" | sqlite3 test.db

sqlite3 dev.db < setup_database.sql
sqlite3 test.db < setup_database.sql