extern mod xmpp;
use xmpp::XmppStream;

fn main() {
    let mut stream = XmppStream::new(~"alice", ~"localhost", ~"test");
    stream.connect();
    stream.handle();
}
