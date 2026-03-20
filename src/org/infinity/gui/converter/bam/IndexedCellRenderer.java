// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bam;

import java.awt.Component;

import javax.swing.DefaultListCellRenderer;
import javax.swing.JList;

/** Adds a prefix to the cell's visual output. */
class IndexedCellRenderer extends DefaultListCellRenderer {
  public IndexedCellRenderer() {
    super();
  }

  @Override
  public Component getListCellRendererComponent(JList<?> list, Object value, int index, boolean isSelected,
      boolean cellHasFocus) {
    String template = "%0" + String.format("%d", Integer.toString(list.getModel().getSize()).length()) + "d - %s";
    return super.getListCellRendererComponent(list, String.format(template, index, value), index, isSelected,
        cellHasFocus);
  }
}