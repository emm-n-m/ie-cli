// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.datatype;

import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionListener;
import java.beans.PropertyChangeEvent;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.util.Objects;

import javax.swing.BorderFactory;
import javax.swing.JButton;
import javax.swing.JComponent;
import javax.swing.JPanel;

import org.infinity.gui.ColorChooser;
import org.infinity.gui.StructViewer;
import org.infinity.gui.ViewerUtil;
import org.infinity.icon.Icons;
import org.infinity.resource.AbstractStruct;

/**
 * Implements a RGB color picker control.
 *
 * <h2>Bean property</h2> When this field is child of {@link AbstractStruct}, then changes of its internal value
 * reported as {@link PropertyChangeEvent}s of the {@link #getParent() parent} struct.
 * <ul>
 * <li>Property name: {@link #getName() name} of this field</li>
 * <li>Property type: {@code int}</li>
 * <li>Value meaning: components of the color in format determined by field</li>
 * </ul>
 */
public class ColorPicker extends Datatype implements Editable, IsNumeric {

  private final ColorChooser.ColorFormat format;
  private final ColorChooser colorChooser;

  private int value;

  /** Initializing color picker with the most commonly used color format {@link Format#XRGB}. */
  public ColorPicker(ByteBuffer buffer, int offset, String name) {
    this(buffer, offset, name, ColorChooser.ColorFormat.ARGB);
  }

  public ColorPicker(ByteBuffer buffer, int offset, String name, ColorChooser.ColorFormat fmt) {
    this(buffer, offset, name, fmt, false);
  }

  public ColorPicker(ByteBuffer buffer, int offset, String name, ColorChooser.ColorFormat fmt, boolean alphaEnabled) {
    super(offset, 4, name);
    format = fmt;
    read(buffer, offset);
    colorChooser = new ColorChooser(format, value, alphaEnabled);
  }

  // --------------------- Begin Interface Editable ---------------------

  @Override
  public JComponent edit(ActionListener container) {
    colorChooser.setBorder(BorderFactory.createEtchedBorder());

    final JButton bUpdate = new JButton("Update value", Icons.ICON_REFRESH_16.getIcon());
    bUpdate.addActionListener(container);
    bUpdate.setActionCommand(StructViewer.UPDATE_VALUE);

    // Setting up main panel
    final JPanel panel = new JPanel(new GridBagLayout());
    final GridBagConstraints gbc = new GridBagConstraints();
    ViewerUtil.setGBC(gbc, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.CENTER, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    panel.add(colorChooser, gbc);
    ViewerUtil.setGBC(gbc, 0, 1, 1, 1, 0.0, 0.0, GridBagConstraints.CENTER, GridBagConstraints.NONE,
        new Insets(4, 0, 0, 0), 0, 0);
    panel.add(bUpdate, gbc);

    panel.setMinimumSize(panel.getPreferredSize());

    return panel;
  }

  @Override
  public void select() {
    colorChooser.resetInitialColor(value);
  }

  @Override
  public boolean updateValue(AbstractStruct struct) {
    setValue(colorChooser.getColor());

    // notifying listeners
    fireValueUpdated(new UpdateEvent(this, struct));

    return true;
  }

  // --------------------- End Interface Editable ---------------------

  // --------------------- Begin Interface Writeable ---------------------

  @Override
  public void write(OutputStream os) throws IOException {
    writeInt(os, value);
  }

  // --------------------- End Interface Writeable ---------------------

  // --------------------- Begin Interface Readable ---------------------

  @Override
  public int read(ByteBuffer buffer, int offset) {
    buffer.position(offset);
    value = buffer.getInt();
    return offset + getSize();
  }

  // --------------------- End Interface Readable ---------------------

  @Override
  public String toString() {
    return String.format("Red: %d, Green: %d, Blue: %d", format.getRed(value), format.getGreen(value),
        format.getBlue(value));
  }

  @Override
  public int hashCode() {
    final int prime = 31;
    int result = super.hashCode();
    result = prime * result + Objects.hash(value);
    return result;
  }

  @Override
  public boolean equals(Object obj) {
    if (this == obj) {
      return true;
    }
    if (!super.equals(obj)) {
      return false;
    }
    if (getClass() != obj.getClass()) {
      return false;
    }
    ColorPicker other = (ColorPicker) obj;
    return value == other.value;
  }

  // --------------------- Begin Interface IsNumeric ---------------------

  @Override
  public long getLongValue() {
    return value & 0xffffffffL;
  }

  @Override
  public int getValue() {
    return value;
  }

  // --------------------- End Interface IsNumeric ---------------------

  private void setValue(int newValue) {
    final int oldValue = value;
    value = newValue;
    if (oldValue != newValue) {
      firePropertyChange(oldValue, newValue);
    }
  }
}
