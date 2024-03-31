let out_element = document.getElementById('js_output');
export function js_log(msg) {
    console.log(msg)
    let p = document.createElement("p");
    p.innerText = msg
    out_element.append(p)
}
document.js_log = js_log
import * as wasm from './public/fern.js';

import { instance } from "@viz-js/viz";
import {EditorView, basicSetup} from "codemirror"
import {javascript} from "@codemirror/lang-javascript"

let lexemes = document.getElementById('lexemes');
let analysis = document.getElementById('analysis');
let viz = await instance();

let spinner = document.getElementById('bottom');
spinner.style.visibility = 'visible'

const ast_canvas = document.getElementById('ast_graph')
const ptree_canvas = document.getElementById('ptree_graph')
var main_ctx = ast_canvas.getContext('2d');
var ptree_ctx = ptree_canvas.getContext('2d');
ast_canvas.width  = ast_canvas.offsetWidth;
ast_canvas.height = ast_canvas.offsetHeight;

ptree_canvas.width  = ptree_canvas.offsetWidth;
ptree_canvas.height = ptree_canvas.offsetHeight;
let ast_graph = render("digraph example3 {}");
let ptree_graph = render("digraph example3 {}");

let parent = document.getElementById('editor');
let myTheme = EditorView.theme({
  "&": {
    fontSize: "16pt"
  },
})

let editor = new EditorView({
  doc: `fn factorial[n: int] {
    if n == 0 {
        return 1;
    }

    return (n * factorial(n - 1));
}
    `,
  extensions: [basicSetup, javascript(), myTheme],
  parent: parent
})


// Adds ctx.getTransform() - returns an SVGMatrix
// Adds ctx.transformedPoint(x,y) - returns an SVGPoint
function trackTransforms(ctx){
    let svg = document.createElementNS("http://www.w3.org/2000/svg",'svg');
    let xform = svg.createSVGMatrix();
    ctx.getTransform = function(){ return xform; };

    let savedTransforms = [];
    let save = ctx.save;
    ctx.save = function(){
        savedTransforms.push(xform.translate(0,0));
        return save.call(ctx);
    };
  
    let restore = ctx.restore;
    ctx.restore = function(){
      xform = savedTransforms.pop();
      return restore.call(ctx);
	      };

    let scale = ctx.scale;
    ctx.scale = function(sx,sy){
      xform = xform.scaleNonUniform(sx,sy);
      return scale.call(ctx,sx,sy);
	      };
  
    let rotate = ctx.rotate;
    ctx.rotate = function(radians){
        xform = xform.rotate(radians*180/Math.PI);
        return rotate.call(ctx,radians);
    };
  
    let translate = ctx.translate;
    ctx.translate = function(dx,dy){
        xform = xform.translate(dx,dy);
        return translate.call(ctx,dx,dy);
    };
  
    let transform = ctx.transform;
    ctx.transform = function(a,b,c,d,e,f){
        let m2 = svg.createSVGMatrix();
        m2.a=a; m2.b=b; m2.c=c; m2.d=d; m2.e=e; m2.f=f;
        xform = xform.multiply(m2);
        return transform.call(ctx,a,b,c,d,e,f);
    };
  
    let setTransform = ctx.setTransform;
    ctx.setTransform = function(a,b,c,d,e,f){
        xform.a = a;
        xform.b = b;
        xform.c = c;
        xform.d = d;
        xform.e = e;
        xform.f = f;
        return setTransform.call(ctx,a,b,c,d,e,f);
    };
  
    let pt  = svg.createSVGPoint();
    ctx.transformedPoint = function(x,y){
        pt.x=x; pt.y=y;
        return pt.matrixTransform(xform.inverse());
    }
}

function compile_code() {
    let output;
    try {
        console.log(editor.state.doc.text.join('\n'))
        console.time("compile code")
        output = wasm.compile_fern(editor.state.doc.text.join('\n'))
        console.timeEnd("compile code")
    } catch(error){
        console.log(error)
        js_log(error)
    }
    output = JSON.parse(output)
    lexemes.innerHTML = output["tokens"]
    return output
}

function render(dot) {
    let svg = viz.renderSVGElement(dot);
    let svgURL = new XMLSerializer().serializeToString(svg);
    let img = new Image();
    img.src = 'data:image/svg+xml; charset=utf8, ' + encodeURIComponent(svgURL);
    return img
}



window.onload = function(){		
    console.log("Setting up")
    trackTransforms(main_ctx);
    trackTransforms(ptree_ctx);

    function redraw(){
      // Clear the entire canvas
      let p1 = main_ctx.transformedPoint(0,0);
      let p2 = main_ctx.transformedPoint(ptree_canvas.width,ptree_canvas.height);
      main_ctx.clearRect(p1.x,p1.y,p2.x-p1.x,p2.y-p1.y);

      main_ctx.save();
      main_ctx.setTransform(1,0,0,1,0,0);
      main_ctx.clearRect(0,0,ptree_canvas.width,ptree_canvas.height);
      main_ctx.restore();

      main_ctx.drawImage(ast_graph,0,0);
    }

    function redraw_ptree(){
      // Clear the entire canvas
      let p1 = ptree_ctx.transformedPoint(0,0);
      let p2 = ptree_ctx.transformedPoint(ptree_canvas.width,ptree_canvas.height);
      ptree_ctx.clearRect(p1.x,p1.y,p2.x-p1.x,p2.y-p1.y);

      ptree_ctx.save();
      ptree_ctx.setTransform(1,0,0,1,0,0);
      ptree_ctx.clearRect(0,0,ptree_canvas.width,ptree_canvas.height);
      ptree_ctx.restore();

      ptree_ctx.drawImage(ptree_graph,0,0);
    }

    let run = () => {
        let spinner = document.getElementById('spinner');
        spinner.style.visibility = 'hidden'
        console.log("spinner", spinner)
        out_element.innerHTML = "";
        analysis.innerHTML = "";

        let output = compile_code()
        ast_graph = render(output["ast"])
        ptree_graph = render(output["ptree"])
        redraw()
        redraw_ptree()
        let analysis_output = output["analysis"]
        for (let i = 0; i < analysis_output.length; i++) {
            let p = document.createElement("p");
            p.innerText = analysis_output[i]
            analysis.append(p)
        }
        console.log("analysis", analysis_output)
        spinner.style.visibility = 'visible'
        console.log("spinner", spinner)
    }    
    console.log("Setting handlers")

    let button = document.getElementById('gen-graph');
    button.addEventListener('click', run, false);

    let ptree = document.getElementById('ptree_tab');
    ptree.addEventListener('click', () => {
        redraw_ptree()
    }, false);
    let ast = document.getElementById('ast_tab');
    ast.addEventListener('click', () => {redraw()}, false);

    let lastX=ast_canvas.width/2, lastY=ast_canvas.height/2;
    let dragStart,dragged;
    ast_canvas.addEventListener('mousedown',function(evt){
        document.body.style.mozUserSelect = document.body.style.webkitUserSelect = document.body.style.userSelect = 'none';
        lastX = evt.offsetX || (evt.pageX - ast_canvas.offsetLeft);
        lastY = evt.offsetY || (evt.pageY - ast_canvas.offsetTop);
        dragStart = main_ctx.transformedPoint(lastX,lastY);
        dragged = false;
    },false);

    ast_canvas.addEventListener('mousemove',function(evt){
        lastX = evt.offsetX || (evt.pageX - ast_canvas.offsetLeft);
        lastY = evt.offsetY || (evt.pageY - ast_canvas.offsetTop);
        dragged = true;
        if (dragStart){
          let pt = main_ctx.transformedPoint(lastX,lastY);
          main_ctx.translate(pt.x-dragStart.x,pt.y-dragStart.y);
          redraw();
        }
    },false);

    ast_canvas.addEventListener('mouseup',function(evt){
        dragStart = null;
        if (!dragged) zoom(evt.shiftKey ? -1 : 1 );
    },false);

    let scaleFactor = 1.1;

    let zoom = function(clicks){
        let pt = main_ctx.transformedPoint(lastX,lastY);
        main_ctx.translate(pt.x,pt.y);
        let factor = Math.pow(scaleFactor,clicks);
        main_ctx.scale(factor,factor);
        main_ctx.translate(-pt.x,-pt.y);
        redraw();
    }

    let handleScroll = function(evt){
        let delta = evt.wheelDelta ? evt.wheelDelta/40 : evt.detail ? -evt.detail : 0;
        if (delta) zoom(delta);
        return evt.preventDefault() && false;
    };
  
    ast_canvas.addEventListener('DOMMouseScroll',handleScroll, false);
    ast_canvas.addEventListener('mousewheel', handleScroll, false);   

    // PTREE GRAPH
    lastX=ptree_canvas.width/2, lastY=ptree_canvas.height/2;
    dragStart,dragged;
    ptree_canvas.addEventListener('mousedown',function(evt){
        document.body.style.mozUserSelect = document.body.style.webkitUserSelect = document.body.style.userSelect = 'none';
        lastX = evt.offsetX || (evt.pageX - ptree_canvas.offsetLeft);
        lastY = evt.offsetY || (evt.pageY - ptree_canvas.offsetTop);
        dragStart = ptree_ctx.transformedPoint(lastX,lastY);
        dragged = false;
    },false);

    ptree_canvas.addEventListener('mousemove',function(evt){
        lastX = evt.offsetX || (evt.pageX - ptree_canvas.offsetLeft);
        lastY = evt.offsetY || (evt.pageY - ptree_canvas.offsetTop);
        dragged = true;
        if (dragStart){
          let pt = ptree_ctx.transformedPoint(lastX,lastY);
          ptree_ctx.translate(pt.x-dragStart.x,pt.y-dragStart.y);
          redraw_ptree();
        }
    },false);

    ptree_canvas.addEventListener('mouseup',function(evt){
        dragStart = null;
        if (!dragged) zoom(evt.shiftKey ? -1 : 1 );
    },false);

    scaleFactor = 1.1;

    zoom = function(clicks){
        let pt = ptree_ctx.transformedPoint(lastX,lastY);
        ptree_ctx.translate(pt.x,pt.y);
        let factor = Math.pow(scaleFactor,clicks);
        ptree_ctx.scale(factor,factor);
        ptree_ctx.translate(-pt.x,-pt.y);
        redraw_ptree();
    }

    handleScroll = function(evt){
        let delta = evt.wheelDelta ? evt.wheelDelta/40 : evt.detail ? -evt.detail : 0;
        if (delta) zoom(delta);
        return evt.preventDefault() && false;
    };
  
    ptree_canvas.addEventListener('DOMMouseScroll',handleScroll, false);
    ptree_canvas.addEventListener('mousewheel', handleScroll, false);
};


await wasm.default();
