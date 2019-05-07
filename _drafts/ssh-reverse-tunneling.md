---
layout: post
---

# Reverse Tunneling with SSH

I had an idea recently. Lots of my devices are locked behind NAT traps, firewalls, or something of the like. The profile on these is typically to allow only a few select inbound ports, if any, and then to entirely block outgoing connections. The end result of this is that if I have a device inside one of these networks, and I want to access it remotely, I cannot.

What if, using some third party like DigitalOcean—whose bread and butter is providing SSHable Linux boxes—one were to establish a reverse tunnel? Using a supervisor, (or better yet, something more simple like a systemd service) one could open an SSH tunnel to that central point, and then crawl back down the chain to get into where they're going.

## How To Do It

Standard SSH clients come with `-L` and `-R` flags. On my MacBook, `man ssh` gets me the following somewhat-helpful documentation:

```
     -L [bind_address:]port:host:hostport
     -L [bind_address:]port:remote_socket
     -L local_socket:host:hostport
     -L local_socket:remote_socket
             Specifies that connections to the given TCP port or Unix socket on the local
             (client) host are to be forwarded to the given host and port, or Unix socket,
             on the remote side.  This works by allocating a socket to listen to either a
             TCP port on the local side, optionally bound to the specified bind_address,
             or to a Unix socket.  Whenever a connection is made to the local port or
             socket, the connection is forwarded over the secure channel, and a connection
             is made to either host port hostport, or the Unix socket remote_socket, from
             the remote machine.

             Port forwardings can also be specified in the configuration file.  Only the
             superuser can forward privileged ports.  IPv6 addresses can be specified by
             enclosing the address in square brackets.

             By default, the local port is bound in accordance with the GatewayPorts set-
             ting.  However, an explicit bind_address may be used to bind the connection
             to a specific address.  The bind_address of ``localhost'' indicates that the
             listening port be bound for local use only, while an empty address or `*'
             indicates that the port should be available from all interfaces.
 
  [...]
  
     -R [bind_address:]port:host:hostport
     -R [bind_address:]port:local_socket
     -R remote_socket:host:hostport
     -R remote_socket:local_socket
     -R [bind_address:]port
             Specifies that connections to the given TCP port or Unix socket on the remote
             (server) host are to be forwarded to the local side.

             This works by allocating a socket to listen to either a TCP port or to a Unix
             socket on the remote side.  Whenever a connection is made to this port or
             Unix socket, the connection is forwarded over the secure channel, and a con-
             nection is made from the local machine to either an explicit destination
             specified by host port hostport, or local_socket, or, if no explicit destina-
             tion was specified, ssh will act as a SOCKS 4/5 proxy and forward connections
             to the destinations requested by the remote SOCKS client.

             Port forwardings can also be specified in the configuration file.  Privileged
             ports can be forwarded only when logging in as root on the remote machine.
             IPv6 addresses can be specified by enclosing the address in square brackets.

             By default, TCP listening sockets on the server will be bound to the loopback
             interface only.  This may be overridden by specifying a bind_address.  An
             empty bind_address, or the address `*', indicates that the remote socket
             should listen on all interfaces.  Specifying a remote bind_address will only
             succeed if the server's GatewayPorts option is enabled (see sshd_config(5)).

             If the port argument is `0', the listen port will be dynamically allocated on
             the server and reported to the client at run time.  When used together with
             -O forward the allocated port will be printed to the standard output.
```

This is arguably not the most concise way of saying it, but here's the gist of what those two options are:

- `-L` opens a tunnel on your machine to the remote machine, and **the tunnel listens on your `-L`ocal machine**.  This means that if you connect to the port you specify on your own machine, SSH will tunnel that traffic through the tunnel it has established, and it'll look like your server is making the requests.
- `-R` opens a tunnel on your machine to the remote machine as before, but this time **the tunnel listens on your `-R`emote server**. This means that if you log into the remote server remotely, and then connect to the port you specified for the tunnel on the other side, SSH will tunnel your traffic back to the machine from whence the tunnel was established.

Can you guess which one is better for this specific use case? `-L` stinks of port proxying, almost like a less-robust SOCKS proxy. (See the `-D` flag in SSH if you want to make a guerilla tunnel through an SSH host) This leaves… `-R`!

### Example

Here's an example. Suppose I have a remote server called `tunl.example.com`, and I want to forward from my laptop, which is behind the NAT or some kind of firewall. This could also be something like a Raspberry Pi or client machine that you want to get into to do upgrades, or something like that, without the fuss of listening on a publicly-accessible port.

On my laptop, this is all I have to run:

```console
$ ssh -R 22222:localhost:22 tunl.example.com
```

Then, to connect, you just connect to `tunl.example.com` and then ssh to port `22222` on `localhost` once you're logged into `tunl.example.com`, à la

```console
$ ssh tunl.example.com
<tunl.example.com> $ ssh -p 22222 localhost
$ # now connected to your laptop!
```

There are some additional, optional parameters available:

- `-f` puts SSH in the background before any commands get run. This is useful if you want SSH to stick around for a while, but you do want SSH to ask you for credentials.
  - You can also pass `-o ExitOnForwardFailure=yes` to make SSH wait until all remote port forwards are successful before backgrounding. This is useful if you want the SSH command to fail if you try to forward to the same port and you already have a forward in progress.
- `-N` tells SSH you don't want to execute a remote command. If all you're doing is forwarding ports, this is handy, as you don't get a pesky shell. (`-f` implies `-n` which has similar behavior, but it's still a good idea to use this flag too.)
  - It's worth noting that this approach is basically encapsulating a tunnel inside of an SSH connection. This means that you can open this tunnel and run some commands on the remote machine, if you want to.
- `-T` _disables_ pseudo-terminal allocation, which you don't want if you just want a background tunnel.

So, all together, this looks like:

```console
$ ssh -fNTR 22222:localhost:22 tunl.example.com
```

Now, if you want to take an extra step into the darkness, and you have port `22222` open on `tunl.example.com`, you can change your `-R` parameter to `22222:0.0.0.0:22`, to bind to `0.0.0.0` on `tunl.example.com`, thereby **forwarding all SSH connections to `22222` on `tunl.example.com` to port `22` on the device you're tunneling from**. Note that 

> Specifying a remote bind_address will only succeed if the server's **GatewayPorts** option is enabled (`see sshd_config(5)`).

This can be pretty dangerous, especially if you're doing some stuff, but it eliminates a step to get back into your laptop.  If you're using PubKey authentication on secured devices and you trust your network connections, though, it should be okay. Here's what that looks like on the laptop

```console
$ ssh -fNTR 22222:0.0.0.0:22 tunl.example.com
$ # <tunnel is now established>
```

and on a remote device,

```
$ ssh -p 22222 <laptop username>@tunl.example.com
$ # <now logged into laptop>
```

### Potential Use Cases

Suppose you have just what my above example was.  A laptop on a home network, an SSH tunnel host, and a work laptop at work behind a corporate firewall.  This would let you SSH, through your publicly-accessible SSH host, back to your home laptop.  A terminal multiplexer like tmux, or adding X Forwarding with `-Y` would let you run graphical apps from your home laptop on your work laptop!  Same would go for a phone, say on LTE or whatnot.  Or, perhaps your laptop is behind a separate NAT and you're on the same network, but you'd like to get to your laptop.  If your laptop can make an outbound connection to your tunnel host, and your phone can make an outbound connection to your tunnel host, then your laptop and your phone can talk together through your tunnel host!

I could also see one messing with the configuration here and trying to get a different utility like Mosh to do the same.

## Conclusion

SSH tunnels are amazing.  They let you get around restrictive firewalls so you can get work done.
