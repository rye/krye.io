---
layout: post
title: "Firewalling with iptables"
subtitle: Cheap, fast, and simple kernel-level Linux firewalls for fun and profit.
---

<blockquote class="disclaimer">
	Disclaimer: Running any of the commands here might break your production systems.
	As is good practice regardless, <i>always</i> test things before using them, and make sure you understand everything that you're using.
	Especially when it's from a random stranger on the Internet.
	You have been warned.
</blockquote>

The netfilter Linux kernel-space APIs that have existed since the early 2010s are a really powerful way of filtering TCP/IP traffic.
Not only can they manage every _packet_ of traffic, they do so at an incredible pace, almost never being the bottleneck.
`iptables` is one common userspace program that allows the interaction with and administration of the netfilter module.

# Intro to `iptables(8)`

The reader may, at this point, be very well familiar with the `iptables` command.
In its most basic form, `iptables` accepts a few key arguments, and can do a few arguments.
In `iptables`-land, operations are centered around the following objects: _chains_, _rules_, and _targets_.
We will now explore each of these to a slight level of detail, however the manual page is unsurprisingly quite helpful here.

- **Chains** are ordered lists of rules that are used to match a set of packets.
- **Rules** _match_ specific packets, and define what to do with those packets.
- **Targets** are destinations that can be used by rules.

Chains, rules, and targets can also belong to specific (routing) _tables_, which are managed by the kernel.
These include the `filter`, `nat`, `mangle`, `raw`, and `security` tables, but we'll just look at the `filter` table in this post.

Chains can also have a "policy," which is just what happens to the packet if it reaches the end of the chain without being matched; it must be either `ACCEPT` or `DROP`.

Users can specify their own rules, chains, and targets, and it is by arranging configurations that users of `iptables` can create robust filtering setups.

Implementations may vary from kernel to kernel, however, as netfilter compilations may have different flags turned on.  You'll want to consult `man 8 iptables-extensions` to make sure you have a given feature.

## Examples

Let's take a look at some chains, now!
Consider the following input sequence:

```
iptables -w --append INPUT --source 1.2.3.4/32 --destination-port 22 --jump ACCEPT
iptables -w       -A INPUT       -s 1.2.3.4/32                           -j DROP
iptables -w       -A INPUT       -s 5.6.7.0/24                           -j DROP
iptables -w       -A INPUT       -s 0.0.0.0/0                            -j ACCEPT
```

This is a list of four rules, all of which modify the `INPUT` chain.

- The first rule,

  ```
  iptables -w --append INPUT --source 1.2.3.4/32 --destination-port 22 --jump ACCEPT
  ```

  appends a rule to the INPUT chain that matches all packets coming from the source `1.2.3.4/32` (on any port) and with the intended destination port `22`. (on any destination host)
  Each packet that matches this will be sent to the `ACCEPT` target, so it will be passed on to userspace.

  In essence, this rule accepts all input from the source `1.2.3.4/32` headed to port `22`.

- The next rule,

  ```
  iptables -w -A INPUT -s 1.2.3.4/32 -j DROP
  ```

  displays one of the powerful corollaries to the chaining model&mdash;any packets with the destination port of 22 will have been accepted by the previous rule in the chain, but if we don't want any additional traffic from `1.2.3.4/32`, the second rule here will drop the rest of the packets.

## Daisy-chaining and logging

If you like the verbiage "chains," you may be pleased to know that targets for rules need not be the defaults, and you can forward packets between chains!
For example,

```
# Create a chain called "log-then-drop"
iptables -w --new-chain LOG_THEN_DROP

# Packets on the LOG_THEN_DROP chain should first be logged, then dropped.
iptables -w -A LOG_THEN_DROP -j LOG  --log-level info
iptables -w -A LOG_THEN_DROP -j DROP

# Send INPUT packets from 1.2.3.4 to the LOG_THEN_DROP chain
iptables -w -A INPUT -s 1.2.3.4/32 -j LOG_THEN_DROP
```

Then `1.2.3.4` would see:

```
$ ping server
PING server.krye.io (X.X.X.X) 56(84) bytes of data.
```

(That is, they would not get ping responses.)

And `server` would see these in the logs:
```
Jun 17 19:37:21 server kernel: IN=eth0 OUT= MAC=[...] SRC=1.2.3.4 DST=X.X.X.X LEN=84 TOS=0x00 PREC=0x00 TTL=63 ID=29283 DF PROTO=ICMP TYPE=8 CODE=0 ID=[...] SEQ=119
```

Using the `LOG` target can be quite helpful for debugging, especially if you're being aggressive such as using a default-`DROP` policy.

## REJECT vs DROP

There seems to be a consensus that `DROP` is better if you want people not to even know that your server exists; any packets that reach the `DROP` target will not provoke a response by the server!
But, sometimes you want to help your users understand what they are supposed to do.
Taking our previous example and making it a bit more friendly,

```
# Delete the DROP rule from above
iptables -w --delete LOG_THEN_DROP -j DROP

# Instead, at the end of our LOG_THEN_DROP chain, reject the packet with "host unreachable"
iptables -w       -A LOG_THEN_DROP -j REJECT --reject-with icmp-host-unreachable
```

Now, `1.2.3.4` instead sees

```
$ ping server
PING server.krye.io (X.X.X.X) 56(84) bytes of data.
From server.krye.io (X.X.X.X) icmp_seq=1 Destination Host Unreachable
From server.krye.io (X.X.X.X) icmp_seq=2 Destination Host Unreachable
From server.krye.io (X.X.X.X) icmp_seq=3 Destination Host Unreachable
From server.krye.io (X.X.X.X) icmp_seq=4 Destination Host Unreachable
From server.krye.io (X.X.X.X) icmp_seq=5 Destination Host Unreachable
From server.krye.io (X.X.X.X) icmp_seq=6 Destination Host Unreachable
```

It's important to note that `DROP` imitates the behavior of pinging a _completely dead_ host, which doesn't help your end-users.
If you want people to know that they are being filtered, it might be advised to instead use the `REJECT` target with an appropriate ICMP message.

<div class="notice wip">
	This post is still a work-in-progress.
	<a href="mailto:kristofer.rye+blog-feedback@gmail.com">Shoot me an email</a> if you have suggestions!
</div>
