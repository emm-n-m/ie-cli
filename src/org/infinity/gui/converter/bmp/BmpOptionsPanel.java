// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bmp;

import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;

import javax.swing.JCheckBox;

import org.infinity.gui.ViewerUtil;
import org.infinity.gui.converter.AbstractConvertPanel;

/** Subpanel with BMP-specific options. */
class BmpOptionsPanel extends AbstractConvertPanel implements ActionListener {
  private JCheckBox cbEnableAlpha;
  private JCheckBox cbFixPremultipliedAlpha;

  public BmpOptionsPanel() {
    super(new GridBagLayout());
    init();
  }

  /** Returns whether the "Enable transparency support" checkbox is selected. */
  public boolean isAlphaSelected() {
    return cbEnableAlpha.isSelected();
  }

  /** Sets whether the "Enable transparency support" checkbox is selected. */
  public void setAlphaSelected(boolean select) {
    cbEnableAlpha.setSelected(select);
  }

  /** Returns whether the "Fix premultiplied alpha" checkbox is enabled and selected. */
  public boolean isPremultipliedAlphaFixSelected() {
    return cbFixPremultipliedAlpha.isEnabled() && cbFixPremultipliedAlpha.isSelected();
  }

  /** Sets whether the "Fix premultiplied alpha" checkbox is selected. Enabled state is handled by the panel. */
  public void setPremultipliedAlphaFixSelected(boolean select) {
    cbFixPremultipliedAlpha.setSelected(select);
  }

  /** Resets the panel to its initial state. */
  public void reset() {
    setAlphaSelected(true);
    setPremultipliedAlphaFixSelected(false);
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == cbEnableAlpha) {
      onEnableAlpha();
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  private void onEnableAlpha() {
    cbFixPremultipliedAlpha.setEnabled(cbEnableAlpha.isSelected());
  }

  private void init() {
    cbEnableAlpha = new JCheckBox("Enable transparency support", true);
    cbEnableAlpha.setToolTipText("Activate to create bitmap files with alpha channel");
    cbEnableAlpha.addActionListener(this);

    cbFixPremultipliedAlpha = new JCheckBox("Fix premultiplied alpha", false);
    cbFixPremultipliedAlpha.setEnabled(cbEnableAlpha.isSelected());
    cbFixPremultipliedAlpha.setToolTipText("Activate if the resulting BMP image differs from the source image");

    final GridBagConstraints c = new GridBagConstraints();

    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    add(cbEnableAlpha, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(4, 0, 0, 0), 0, 0);
    add(cbFixPremultipliedAlpha, c);
  }
}