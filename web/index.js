import * as wasm from './public/fern.js';
import * as canvas_area from './canvas-area.js';
import { instance } from "@viz-js/viz";
import {EditorView, basicSetup} from "codemirror"
import {EditorState} from "@codemirror/state"
import {javascript} from "@codemirror/lang-javascript"

var parent = document.getElementById('editor');
let editor = new EditorView({
  doc: `fn main() {
  let x = 0;
  if x > 0 {
    return 0;
  } else {
    return 1
  }
}
    `,
  extensions: [basicSetup, javascript()],
  parent: parent
})


let viz = await instance();
await wasm.default();

var button = document.getElementById('gen-graph');
var message = document.getElementById('err');

var ca = document.getElementById('graph-container');
function compile_code() {
  try{
    console.log(editor.state.doc.text.join('\n'))
    console.time("compile code")
    let dot = wasm.compile_fern(editor.state.doc.text.join('\n'))
    console.timeEnd("compile code")

    let canvas = document.getElementById('graph')
    let ctx = canvas.getContext("2d")
    let view = ca.view
    let w = ctx.canvas.width, h = ctx.canvas.height, 
            sz = 20*view.scl, xoff = view.x%sz, yoff = view.y%sz;
    if (view.scl > 0.2) {
      ctx.strokeStyle = "#ccc";
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let x=xoff,nx=w+1; x<nx; x+=sz) { ctx.moveTo(x,0); ctx.lineTo(x,h); }
      for (let y=yoff,ny=h+1; y<ny; y+=sz) { ctx.moveTo(0,y); ctx.lineTo(w,y); }
      ctx.stroke();
      ctx.strokeStyle = "#555"; // origin lines ...
      ctx.beginPath();
      ctx.moveTo(view.x,0); ctx.lineTo(view.x,h);
      ctx.moveTo(0,view.y); ctx.lineTo(w,view.y);
      ctx.stroke();
    }

    // let svg = viz.renderSVGElement(dot);
    // renderHtmlToCanvas(ctx, svg);
    } catch(error){
    console.log(error)
    message.innerHTML= "Fail!"
  }
  message.innerHTML= "Success!"
}

document.onload = () => {
    // ca.on('pointer', e => {
    //         let {x,y} = ca.pntToUsr(e), 
    //             prec = Math.max(Math.log(ca.view.scl)/Math.log(2), 0);
    //         coords.innerHTML = `pos: ${(x).toFixed(prec)} / ${(y).toFixed(prec)}`;} )
    //   .on('resize', e => {
    //         size.innerHTML = `size: ${(e.width).toFixed()} / ${(e.height).toFixed()}`; });

    // ca.resize({width,height}=ca);  // cause single initial notification ..
    // compile_code()
  console.log("loaded")
}

button.addEventListener('click', compile_code, false);

function renderHtmlToCanvas(ctx, svg_element) {
  var svgURL = new XMLSerializer().serializeToString(svg_element);
  var img = new Image();
  img.onload = function() {
    ctx.drawImage(this, 0, 0);
  }
  img.src = 'data:image/svg+xml; charset=utf8, ' + encodeURIComponent(svgURL);
}
