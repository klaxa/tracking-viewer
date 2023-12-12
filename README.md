Track yo'self 2 viewer
======================

This GUI tool is a companion project of [klaxa/tracking_2](https://github.com/klaxa/tracking_2).

As with the other programs the location of the database defaults to
`tracking.db` in the current directory. This value can either be set
with the `-d` flag or by setting the environment variable `TRACKING_DB`
to the path you wish to use.

The design takes inspiration from the gnome-calendar and other similar
software. In general a similar type of output as `gen_chart` generates
it is produced:

Example view zoomed out:

![zoomed_out](https://github.com/klaxa/tracking-viewer/assets/1451995/76d0a2b7-4d1c-453d-996e-a0988700553c)

Example view zoomed in:

![zoomed_in](https://github.com/klaxa/tracking-viewer/assets/1451995/80f8c383-f868-4cbb-929f-9d656f2f8cc8)


Currently there are a few limitations, for example the selection range
is always set to the current month. Another current limitation is the
handling of daylight savings transitions.

To change the selected month use the month overview in the top left
corner. Clicking on any day will change the selection to that month.
Clicking on a day will also change the week display to display the week
that day is part of (starting on Monday).

The week view can also be zoomed in. To do so simply click and scroll.
Currently the position in the view is not mantained when zooming.
