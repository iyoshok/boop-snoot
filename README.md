# BOOP Client
**Disclaimer:** This project is the result of a personal coding excercise with the goal of improving my familiarity with the [Tauri application framework](https://tauri.studio). 
This means that I knowingly ignored best practices in software, protocol and interface design, and avoided safety guardrails, if necessary, to speed up the
developement process. I don't intend on actively developing this system any further. However, if you want to contribute to this project, feel free to submit
pull requests or open issues, I will try to reply to them as quickly as possible. 

## BOOP
- Server: https://github.com/iyoshok/boop-relay
- Client: <- You are here

This repository contains the code for the BOOP client application. It's basically a small desktop application that allows you to ~poke~ boop your friends, 
partners or colleagues (if you have that kind of relationship). As stated in the disclaimer, this is just a tiny side project and, as such, there's no elaborate
registration or account creation process. If you want to use the service, your server provider has to add your username and a hash of your desired password 
into their configuration file and that's it. No Database or anything. Yes you have to restart the server every time you change client credentials.

After your "account registration", you just have to enter the server address and your login credentials into the client and register your partners 
usernames locally (oh yeah did I mention there's no discovery feature or anything? you can only boop someone if you know their username).
But you can pick cool nicknames for them, those nicknames are only local settings tho, so if you delete your installation, you'll have to recreate them.
Also, their client will only display the boop if they've registered you as boop partner as well (because consent is key 😊).

The interface is minimal and self-explanatory, if you disagree with the latter, please open an issue and I will try to provide additional documentation
or streamline confusing processes.

Happy Booping!
