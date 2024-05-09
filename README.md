# Rust SQL Mini Server
A simple SQL server supporting SELECT, INSERT, DELETE, where SELECT and DELETE clause supports the use of hash indexes.
The storage of the data is persistent so there is no loss data when server is turned off. The reading and writing to files is
synchronized using read write lock.