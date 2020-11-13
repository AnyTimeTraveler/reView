// Found by finding the maximum values when experimenting
const MAX_W = 1872;
const MAX_H = 1404;

// landscape / portrait
let rotate = false;

let canvas = document.getElementById("canvas");
let ctx = canvas.getContext("2d");

function connect() {
    let address = document.getElementById("address").value;

    document.getElementById("status").textContent = "connecting...";
    console.log("Attempting to connect to ws://" + address);
    let websocket = new WebSocket("ws://" + address);
    websocket.binaryType = 'arraybuffer';

    websocket.onopen = function () {
        console.log("Connected");
        document.getElementById("status").textContent = "connected";
    }

    websocket.onerror = function () {
        console.log("Error");
        document.getElementById("status").textContent = "error";
    }

    websocket.onclose = function () {
        console.log("Disconnected");
        if (document.getElementById("status").textContent !== "error") {
            document.getElementById("status").textContent = "disconnected";
        }
    }

    websocket.onmessage = draw;
}

function draw(event) {
    const data = event.data;
    const dv = new DataView(data);
    canvas.dataset = null;
    let image_data = ctx.createImageData(MAX_W, MAX_W);

    for (let i = 0; i < dv.byteLength; i++) {
        let h = dv.getUint16(i);
        i += 2;
        let w = dv.getUint16(i);
        i += 2;
        for (; dv.getUint8(i) !== 255; i++) {
            image_data[h * MAX_W + w] = dv.getUint8(i);
        }
    }
}

// document.getElementById("address").value = window.location.hostname + ":55555";
// connect();