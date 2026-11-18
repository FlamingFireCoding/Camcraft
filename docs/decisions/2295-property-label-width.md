## #2295 - Use available width for property labels

**Decided:** Give the shared label cell half of the available row width.
Keep the minimum label width when space permits, and reserve space for a compact control.
Use the same label cell for property rows and checkbox rows.
Keep egui's full-name tooltip when a label must be cut off.

**Why:** The fixed 120-point column cuts off names while the panel has free space.
Responsive columns give labels more space and keep controls aligned.
The hover text gives access to the full name in a narrow panel.

**Considered and dropped:** A larger fixed column can crowd controls in a narrow panel.
Changes in each property page repeat the same layout rule.
Labels on multiple lines change row heights and reduce the number of visible properties.

**Cost to change later:** Small. Change the width calculation in the shared label cell and update its layout tests.
