
await Bun.build({
  entrypoints: [
    './compiler.js', 
  ],
  outdir: './public',
})
