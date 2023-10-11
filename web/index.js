import * as wasm from './public/fern.js';
import { instance } from "@viz-js/viz";
import {EditorView, basicSetup} from "codemirror"
import {EditorState} from "@codemirror/state"
import {javascript} from "@codemirror/lang-javascript"

var parent = document.getElementById('editor');
let editor = new EditorView({
  doc: "console.log('hello')\n",
  extensions: [basicSetup, javascript()],
  parent: parent
})


let viz = await instance();
await wasm.default();
const start = Date.now();


var button = document.getElementById('gen-graph');
var message = document.getElementById('err');
button.addEventListener('click', function() {
  try{
    console.log(editor.state.doc.text.join('\n'))
    let dot = wasm.compile_fern(editor.state.doc.text.join('\n'))
    const end = Date.now();
    console.log(`Execution time: ${end - start} ms`);

    let graph = document.getElementById('graph')
    graph.innerHTML = '';
    graph.appendChild(viz.renderSVGElement(dot))
  } catch(error){
    message.innerHTML= "Fail!"
  }
  message.innerHTML= "Success!"
}, false);
