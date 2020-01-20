# Meh-rs

This is a binary application for consuming the (Meh.com)[https://meh.com/forum/topics/meh-api] API.

Currently the functionality is to just ping for a new deal, using the expiration of the current
deal to know when to wake-up and ping again.

The program will display a 3 line progress spinner about what the current deal is (title and $)
The url of the current deal and the next check.

```
▒ Bag of Crap (5.00)
▒ https://meh.com/deals/bag-of-crap
▒ checking again in 12 hours at 10:00 pm
```

If a file name "alert.ps1" on windows or "alert.sh" on a unix-like os exists, the program will attempt to execute this file when a new deal is detected.

