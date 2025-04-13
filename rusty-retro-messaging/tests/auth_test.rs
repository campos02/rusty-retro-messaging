use core::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::test]
async fn login() {
    let mut socket = TcpStream::connect("127.0.0.1:1863").await.unwrap();
    let (mut rd, mut wr) = socket.split();
    let mut buf = vec![0; 1664];

    wr.write_all(b"VER 1 MSNP11 CVR0\r\n").await.unwrap();
    let received = rd.read(&mut buf).await.unwrap();
    let message = str::from_utf8(&buf[..received]).unwrap();

    assert_eq!(message, "VER 1 MSNP11\r\n");

    wr.write_all(b"CVR 2 0x040c winnt 5.1 i386 MSNMSGR 7.0.0777 msmsgs alice@hotmail.com\r\n")
        .await
        .unwrap();
    let received = rd.read(&mut buf).await.unwrap();
    let message = str::from_utf8(&buf[..received]).unwrap();

    assert_eq!(
        message,
        "CVR 2 1.0.0000 1.0.0000 7.0.0425 http://download.microsoft.com/download/D/F/B/DFB59A5D-92DF-4405-9767-43E3DF10D25B/fr/Install_MSN_Messenger.exe http://messenger.msn.com/fr\r\n"
    );

    wr.write_all(b"USR 3 TWN I alice@hotmail.com\r\n")
        .await
        .unwrap();
    let received = rd.read(&mut buf).await.unwrap();
    let message = str::from_utf8(&buf[..received]).unwrap();

    assert_eq!(
        message,
        "USR 3 TWN S ct=1,rver=1,wp=FS_40SEC_0_COMPACT,lc=1,id=1\r\n"
    );
}
