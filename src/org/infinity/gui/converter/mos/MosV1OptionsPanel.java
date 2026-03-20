// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.mos;

import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;

import javax.swing.JCheckBox;
import javax.swing.JPanel;

import org.infinity.gui.ViewerUtil;
import org.infinity.gui.converter.AbstractConvertPanel;

/** Subpanel with MOS V1-specific options. */
class MosV1OptionsPanel extends AbstractConvertPanel {
  private JCheckBox cbCompressed;

  public MosV1OptionsPanel() {
    super(new GridBagLayout());
    init();
  }

  /** Returns whether the "Compressed" checkbox is selected. */
  public boolean isCompressed() {
    return cbCompressed.isSelected();
  }

  /** Sets the selection state of the "Compressed" checkbox. */
  public void setCompressed(boolean set) {
    cbCompressed.setSelected(set);
  }

  /** Resets the panel to its initial state. */
  public void reset() {
    setCompressed(false);
  }

  private void init() {
    cbCompressed = new JCheckBox("Compressed (MOSC)");

    final GridBagConstraints c = new GridBagConstraints();

    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    add(cbCompressed, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.BOTH,
        new Insets(0, 0, 0, 0), 0, 0);
    add(new JPanel(), c);

    reset();
  }
}