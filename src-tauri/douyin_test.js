const regex = /<script[^>]*?>window\._ROUTER_DATA\s*=\s*(.*?)<\/script>/s;
fetch("https://www.douyin.com/share/video/7464082269229042971", {
    headers: {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"
    }
}).then(r => r.text()).then(t => {
    console.log(t.substring(0, 100)); // to see if it's the challenge page
    let m = t.match(regex);
    if(m) {
        console.log("MATCH DATA FOUND length:", m[1].length);
    } else {
        console.log("NO _ROUTER_DATA. Other scripts:");
        let scripts = [...t.matchAll(/<script[^>]*?>(.*?)<\/script>/gs)];
        for(let s of scripts) {
            let src = s[1].substring(0, 50);
            if(src.includes('window.')) console.log(src);
        }
    }
});
