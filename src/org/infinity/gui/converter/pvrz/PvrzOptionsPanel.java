// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.pvrz;

import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;

import javax.swing.JButton;
import javax.swing.JComboBox;
import javax.swing.JLabel;
import javax.swing.JOptionPane;

import org.infinity.gui.ViewerUtil;
import org.infinity.gui.converter.AbstractConvertPanel;

/** Subpanel with Pvrz-specific options. */
class PvrzOptionsPanel extends AbstractConvertPanel implements ActionListener {
  private JComboBox<CompressionType> cbCompression;
  private JButton bCompressionHelp;

  public PvrzOptionsPanel() {
    super(new GridBagLayout());
    init();
  }

  /** Returns the currently selected compression type. */
  public CompressionType getCompressionType() {
    final int index = cbCompression.getSelectedIndex();
    if (index >= 0) {
      return cbCompression.getModel().getElementAt(index);
    } else {
      return CompressionType.AUTO;
    }
  }

  /** Sets the currently selected compression type. */
  public void setCompressionType(CompressionType type) {
    if (type == null) {
      type = CompressionType.AUTO;
    }
    cbCompression.setSelectedItem(type);
  }

  /** Resets the panel to its initial state. */
  public void reset() {
    setCompressionType(null);
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == bCompressionHelp) {
      JOptionPane.showMessageDialog(this, CompressionType.getHelp(), "About Compression Types",
          JOptionPane.INFORMATION_MESSAGE);
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  private void init() {
    final JLabel lCompression = new JLabel("Compression type:");

    cbCompression = new JComboBox<>(CompressionType.values());
    cbCompression.setSelectedIndex(0);

    bCompressionHelp = new JButton("?");
    bCompressionHelp.setToolTipText("About compression types");
    bCompressionHelp.addActionListener(this);
    final Insets insets = bCompressionHelp.getMargin();
    insets.left = insets.right = 4;
    bCompressionHelp.setMargin(insets);

    final GridBagConstraints c = new GridBagConstraints();

    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    add(lCompression, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 4, 0, 0), 0, 0);
    add(cbCompression, c);
    ViewerUtil.setGBC(c, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 4, 0, 0), 0, 0);
    add(bCompressionHelp, c);

    reset();
  }
}