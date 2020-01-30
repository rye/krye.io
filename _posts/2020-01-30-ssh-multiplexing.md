---
layout: post
title: 'SSH Multiplexing'
subtitle: 'Eliminating Connection Overhead and Saving Time'
---

Let's suppose you make anywhere from tens to thousands of ssh connections a day.
You're security-minded, so you crank all your key sizes up to the maximum.
But now, you've run into a problem!
SSH connections are _expensive_!
Not only are TCP connections expensive, but the overhead of negotiating secure connection scales linearly with the amount of traffic that has to flow over the pipe _during_ that negotiation process, and as a result your keys (of all things!) have begun to slow down your workflow.
You begin to feel like having all these different concurrent _and_ recurrent connections can somehow be optimized...

And that's where multiplexing---connection sharing---comes in.

# Configuration

In your SSH configuration file (`~/.ssh/config`) you can do something like the following to enable connection sharing.

```
Host myserver
	# [...]
	ControlMaster auto
	ControlPath ~/.ssh/ctl.%r@%h:%p
	ControlPersist 15m
	# [...]
```

Let's break that down, just in case you're not familiar.
All of this is summarized from [the manual page][ssh_config].

- **`Host myserver`** is the beginning of a block of configuration corresponding to one specific host, `myserver`.
  The subsequent configuration directives we give (up until the next `Host` or `Match` block) will be associated with connections to `myserver`.
  Important to note is that these directives are generally only matched against the `hostname` argument when you run `ssh` from the command-line.
  So, something like `ssh myserver.example.com` would _not_ match this configuration block, since your configuration is only for `myserver`.
  (See [`CanonicalizeHostname`](https://man.openbsd.org/ssh_config#CanonicalizeHostname) if you want to change that.)

- **`ControlMaster`** controls the sharing of multiple connections over a single network connection.
  If you set it to `yes`, `ssh` will listen for connections on the control socket given by `ControlPath`, and additional connections should have `ControlMaster` set to `no`, since they are instead to use the control socket.
  Here, we've set `ControlMaster` to `auto`, which will _try_ to use a master connection if one already exists, but fall back to creating a new one if it doesn't already exist.

- **`ControlPath`** controls _where_ to locate the control socket that will be used by other connections.
  I tend to keep all my SSH-related goodies in `~/.ssh`, but you could put this in volatile storage if you wanted.

  You can use any valid `ssh_config` [tokens](https://man.openbsd.org/ssh_config#TOKENS) when building your `ControlPath`.
  As of the time of writing, these include

  - `%r`, which is substituted with the remote username,
  - `%h`, which is substituted with the remote hostname,
  - `%p`, which is substituted with the remote port,
  - `%l`, which is substituted with the local hostname, including the domain name, and
  - (my favorite) `%C`, which is a hash of `%l%h%p%r`.

- **`ControlPersist`** allows the master `ssh` process, once it has terminated, to remain idle for the specified amount of time (or, indefinitely if the given parameter is `yes` or `0`) with no client connections.
  This means that---in the example given above---the `ControlMaster` process will stick around for `15m` before closing the socket and shutting down.

[ssh_config]: https://man.openbsd.org/ssh_config

Now, if you have a running `ControlMaster` and want to check on its status or stop it, you can use `ssh -O` to send a message.
For example, `ssh -O check myserver` or `ssh -O stop myserver`, respectively.

# Performance

Since all of the TCP connection overhead is handled by one single master process, you can drastically reduce the amount of time you spend waiting on SSH connections from day to day.
That's not even mentioning key exchange, which is a far bigger overhead!
With my 16384-bit RSA keys, I can end up spending quite a bit of time waiting for connections to open.
In fact, that gets _very_ annoying when I'm trying to do lots of SSH operations in a short period of time.

This is also very useful if you use Git over SSH!
Your subsequent Git operations do not have to wait for a new connection to get established, which makes things faster.
Consider the following SSH session logs, for comparison of the overhead reduction.

```console
$ time ssh -v -F /dev/null -T git@github.com
OpenSSH_8.1p1, OpenSSL 1.1.1d  10 Sep 2019
debug1: Reading configuration data /dev/null
debug1: Connecting to github.com [140.82.113.4] port 22.
debug1: Connection established.
debug1: identity file /Users/kristofer/.ssh/id_rsa type 0
debug1: identity file /Users/kristofer/.ssh/id_rsa-cert type -1
debug1: identity file /Users/kristofer/.ssh/id_dsa type -1
debug1: identity file /Users/kristofer/.ssh/id_dsa-cert type -1
debug1: identity file /Users/kristofer/.ssh/id_ecdsa type -1
debug1: identity file /Users/kristofer/.ssh/id_ecdsa-cert type -1
debug1: identity file /Users/kristofer/.ssh/id_ed25519 type -1
debug1: identity file /Users/kristofer/.ssh/id_ed25519-cert type -1
debug1: identity file /Users/kristofer/.ssh/id_xmss type -1
debug1: identity file /Users/kristofer/.ssh/id_xmss-cert type -1
debug1: Local version string SSH-2.0-OpenSSH_8.1
debug1: Remote protocol version 2.0, remote software version babeld-1f0633a6
debug1: no match: babeld-1f0633a6
debug1: Authenticating to github.com:22 as 'git'
debug1: SSH2_MSG_KEXINIT sent
debug1: SSH2_MSG_KEXINIT received
debug1: kex: algorithm: curve25519-sha256
debug1: kex: host key algorithm: rsa-sha2-512
debug1: kex: server->client cipher: chacha20-poly1305@openssh.com MAC: <implicit> compression: none
debug1: kex: client->server cipher: chacha20-poly1305@openssh.com MAC: <implicit> compression: none
debug1: expecting SSH2_MSG_KEX_ECDH_REPLY
debug1: Server host key: ssh-rsa [REDACTED]
debug1: Host 'github.com' is known and matches the RSA host key.
debug1: Found key in /Users/kristofer/.ssh/known_hosts:134
debug1: rekey out after 134217728 blocks
debug1: SSH2_MSG_NEWKEYS sent
debug1: expecting SSH2_MSG_NEWKEYS
debug1: SSH2_MSG_NEWKEYS received
debug1: rekey in after 134217728 blocks
debug1: Will attempt key: /Users/kristofer/.ssh/id_rsa RSA [REDACTED]
debug1: Will attempt key: /Users/kristofer/.ssh/id_dsa
debug1: Will attempt key: /Users/kristofer/.ssh/id_ecdsa
debug1: Will attempt key: /Users/kristofer/.ssh/id_ed25519
debug1: Will attempt key: /Users/kristofer/.ssh/id_xmss
debug1: SSH2_MSG_EXT_INFO received
debug1: kex_input_ext_info: server-sig-algs=<ssh-ed25519,ecdsa-sha2-nistp256,ecdsa-sha2-nistp384,ecdsa-sha2-nistp521,ssh-rsa,rsa-sha2-512,rsa-sha2-256,ssh-dss>
debug1: SSH2_MSG_SERVICE_ACCEPT received
debug1: Authentications that can continue: publickey
debug1: Next authentication method: publickey
debug1: Offering public key: /Users/kristofer/.ssh/id_rsa RSA [REDACTED]
debug1: Server accepts key: /Users/kristofer/.ssh/id_rsa RSA [REDACTED]
debug1: Authentication succeeded (publickey).
Authenticated to github.com ([140.82.113.4]:22).
debug1: channel 0: new [client-session]
debug1: Entering interactive session.
debug1: pledge: network
debug1: client_input_channel_req: channel 0 rtype exit-status reply 0
Hi rye! You've successfully authenticated, but GitHub does not provide shell access.
debug1: channel 0: free: client-session, nchannels 1
Transferred: sent 8008, received 4012 bytes, in 0.1 seconds
Bytes per second: sent 77931.4, received 39043.6
debug1: Exit status 1
        0.95 real         0.45 user         0.01 sys
```

Here we used `-F /dev/null` to avoid having our configuration read, which forces us to create a new SSH connection and not use multiplexing.
Compare that to this, with multiplexing enabled:

```console
$ ssh -v -T git@github.com
OpenSSH_8.1p1, OpenSSL 1.1.1d  10 Sep 2019
debug1: Reading configuration data /Users/kristofer/.ssh/config
debug1: /Users/kristofer/.ssh/config line 39: Applying options for github.com
debug1: Reading configuration data /usr/local/etc/ssh/ssh_config
debug1: auto-mux: Trying existing master
debug1: Control socket "/Users/kristofer/.ssh/ctl.git@github.com:22" does not exist
debug1: Connecting to github.com [192.30.253.112] port 22.
debug1: Connection established.
debug1: identity file /Users/kristofer/.ssh/id_rsa type 0
debug1: identity file /Users/kristofer/.ssh/id_rsa-cert type -1
debug1: identity file /Users/kristofer/.ssh/id_dsa type -1
debug1: identity file /Users/kristofer/.ssh/id_dsa-cert type -1
debug1: identity file /Users/kristofer/.ssh/id_ecdsa type -1
debug1: identity file /Users/kristofer/.ssh/id_ecdsa-cert type -1
debug1: identity file /Users/kristofer/.ssh/id_ed25519 type -1
debug1: identity file /Users/kristofer/.ssh/id_ed25519-cert type -1
debug1: identity file /Users/kristofer/.ssh/id_xmss type -1
debug1: identity file /Users/kristofer/.ssh/id_xmss-cert type -1
debug1: Local version string SSH-2.0-OpenSSH_8.1
debug1: Remote protocol version 2.0, remote software version babeld-1f0633a6
debug1: no match: babeld-1f0633a6
debug1: Authenticating to github.com:22 as 'git'
debug1: SSH2_MSG_KEXINIT sent
debug1: SSH2_MSG_KEXINIT received
debug1: kex: algorithm: curve25519-sha256
debug1: kex: host key algorithm: rsa-sha2-512
debug1: kex: server->client cipher: chacha20-poly1305@openssh.com MAC: <implicit> compression: none
debug1: kex: client->server cipher: chacha20-poly1305@openssh.com MAC: <implicit> compression: none
debug1: expecting SSH2_MSG_KEX_ECDH_REPLY
debug1: Server host key: ssh-rsa [REDACTED]
debug1: Host 'github.com' is known and matches the RSA host key.
debug1: Found key in /Users/kristofer/.ssh/known_hosts:134
debug1: rekey out after 134217728 blocks
debug1: SSH2_MSG_NEWKEYS sent
debug1: expecting SSH2_MSG_NEWKEYS
debug1: SSH2_MSG_NEWKEYS received
debug1: rekey in after 134217728 blocks
debug1: Will attempt key: /Users/kristofer/.ssh/id_rsa RSA [REDACTED]
debug1: Will attempt key: /Users/kristofer/.ssh/id_dsa
debug1: Will attempt key: /Users/kristofer/.ssh/id_ecdsa
debug1: Will attempt key: /Users/kristofer/.ssh/id_ed25519
debug1: Will attempt key: /Users/kristofer/.ssh/id_xmss
debug1: SSH2_MSG_EXT_INFO received
debug1: kex_input_ext_info: server-sig-algs=<ssh-ed25519,ecdsa-sha2-nistp256,ecdsa-sha2-nistp384,ecdsa-sha2-nistp521,ssh-rsa,rsa-sha2-512,rsa-sha2-256,ssh-dss>
debug1: SSH2_MSG_SERVICE_ACCEPT received
debug1: Authentications that can continue: publickey
debug1: Next authentication method: publickey
debug1: Offering public key: /Users/kristofer/.ssh/id_rsa RSA [REDACTED]
debug1: Server accepts key: /Users/kristofer/.ssh/id_rsa RSA [REDACTED]
debug1: Authentication succeeded (publickey).
Authenticated to github.com ([192.30.253.112]:22).
debug1: setting up multiplex master socket
debug1: channel 0: new [/Users/kristofer/.ssh/ctl.git@github.com:22]
debug1: control_persist_detach: backgrounding master process
debug1: forking to background
debug1: Entering interactive session.
debug1: pledge: id
debug1: multiplexing control connection
debug1: channel 1: new [mux-control]
debug1: channel 2: new [client-session]
debug1: client_input_channel_req: channel 2 rtype exit-status reply 0
Hi rye! You've successfully authenticated, but GitHub does not provide shell access.
debug1: channel 2: free: client-session, nchannels 3
debug1: channel 1: free: mux-control, nchannels 2
        0.96 real         0.45 user         0.01 sys
```

...Wait, that looks the same!
Except now we have some `channel #: [...]` messages.
What happens if we run this again?

```console
$ time ssh -v -T git@github.com
OpenSSH_8.1p1, OpenSSL 1.1.1d  10 Sep 2019
debug1: Reading configuration data /Users/kristofer/.ssh/config
debug1: /Users/kristofer/.ssh/config line 39: Applying options for github.com
debug1: Reading configuration data /usr/local/etc/ssh/ssh_config
debug1: auto-mux: Trying existing master
debug1: multiplexing control connection
debug1: channel 1: new [mux-control]
debug1: channel 2: new [client-session]
debug1: client_input_channel_req: channel 2 rtype exit-status reply 0
Hi rye! You've successfully authenticated, but GitHub does not provide shell access.
debug1: channel 2: free: client-session, nchannels 3
debug1: channel 1: free: mux-control, nchannels 2
        0.12 real         0.00 user         0.00 sys
```

Ah ha!
As you can see, we cut down the entire transaction time from `0.96` seconds to `0.12` seconds!
(Only one run, so those numbers could be complete flukes.)

The point remains, and hopefully is illustrated by my verbose logs here.
The _second_ connection (and all subsequent connections before the master process decides to stop) took _far less_ time to establish and get going.
This is because all the key exchange and session-related bits were already done!
All this means we can get more done with less connections.

# Conclusion

If you use a lot of SSH in your day-to-day, and you consistently open many connections to the same machine, you might want to try out multiplexing!
You can also build a lot of interesting things by combining this with [reverse tunneling](https://man.openbsd.org/ssh#R), too.
Maybe that'll be another story for another time. :)
