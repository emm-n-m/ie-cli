// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter;

import java.awt.Component;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;

import javax.swing.JPanel;

import org.infinity.gui.ViewerUtil;

/**
 * A thin wrapper for custom options panels for conversion operations. (Currently used by MOS and TIS conversion
 * dialogs.)
 */
public class ConvertOptionsPanel extends AbstractConvertPanel {

  private final Component optionsPanel;

  public ConvertOptionsPanel(String title, Component optionsPanel) {
    super(new GridBagLayout(), title);
    this.optionsPanel = optionsPanel;
    init();
  }

  /**
   * Returns the options panel that was passed to the constructor. Returns {@code null} if no custom options panel was
   * specified.
   */
  public Component getOptionsPanel() {
    return optionsPanel;
  }

  private void init() {
    final Component component = (optionsPanel != null) ? optionsPanel : new JPanel();

    final GridBagConstraints c = new GridBagConstraints();
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 4, 4, 4), 0, 0);
    add(component, c);
  }
}
