# Static Segoe UI runtime fonts

`Segoe UI.ttf` and `Segoe UI Bold.ttf` are full static instances generated from
`../SegoeUIVF.ttf`. BasScript loads these files at runtime and embeds them in
Markdown PDF exports; it does not load the variable source.

The instances use the variable font's named Text optical size:

```sh
python -m fontTools.varLib.instancer ../SegoeUIVF.ttf \
  wght=400 opsz=10.5 --update-name-table --no-recalc-timestamp \
  -o 'Segoe UI.ttf'
python -m fontTools.varLib.instancer ../SegoeUIVF.ttf \
  wght=700 opsz=10.5 --update-name-table --no-recalc-timestamp \
  -o 'Segoe UI Bold.ttf'
```

FontTools 4.59.1 was used for the checked-in instances. The source has weight
and optical-size axes, but no italic axis, so `Segoe UI Italic.ttf` and
`Segoe UI Bold Italic.ttf` remain the existing genuine static italic faces.
