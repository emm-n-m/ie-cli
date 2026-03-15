// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

import java.awt.event.KeyEvent;
import java.awt.event.KeyListener;
import java.awt.event.MouseEvent;
import java.awt.event.MouseListener;
import java.math.BigDecimal;
import java.text.NumberFormat;
import java.text.ParseException;
import java.util.EnumMap;
import java.util.Map;
import java.util.Objects;
import java.util.Set;

import javax.swing.JFormattedTextField;
import javax.swing.SwingUtilities;
import javax.swing.text.NumberFormatter;

import org.infinity.util.Misc;

/**
 * Helper class that associates {@link JFormattedTextField}s with enum values and provides several
 * utility functions.

 * @param <E> Enum class to use as key. The enum type must implement the {@link InputBounds} interface.
 */
public class InputTracker<E extends Enum<E>> implements MouseListener, KeyListener {
  private final EnumMap<E, JFormattedTextField> controls;

  public InputTracker(Class<E> enumClass) {
    if (!InputBounds.class.isAssignableFrom(enumClass)) {
      throw new IllegalArgumentException("Input tracker argument is not compatible.");
    }
    this.controls = new EnumMap<>(enumClass);
  }

  /** Links a {@code key} enum to a {@link JFormattedTextField} instance from the dialog. */
  public E register(E key, JFormattedTextField value) {
    if (key != null && value != null) {
      controls.putIfAbsent(key, value);
    }
    return key;
  }

  /** Removes the {@link JFormattedTextField} instance linked to the specified {@code key} enum. */
  public E unregister(E key) {
    if (key != null) {
      controls.remove(key);
    }
    return key;
  }

  /** Returns the {@link JFormattedTextField} instance linked to the specified {@code key} enum. */
  public JFormattedTextField get(E key) {
    JFormattedTextField retVal = null;
    if (key != null) {
      retVal = controls.get(key);
    }
    return retVal;
  }

  /** Returns a {@link Set} view of all enums linked to {@link JFormattedTextField} instances. */
  public Set<E> getControls() {
    return controls.keySet();
  }

  /**
   * Returns the numeric value currently held by the specified text field control.
   * Returns {@code null} if the input could not be converted to a number.
   */
  public Number getValue(E key) {
    Number retVal = null;
    if (key != null) {
      final JFormattedTextField tf = controls.get(key);
      if (tf != null) {
        try {
          if (tf.getFormatter() instanceof NumberFormatter) {
            final NumberFormatter formatter = (NumberFormatter)tf.getFormatter();
            if (formatter.getFormat() instanceof NumberFormat) {
              final NumberFormat format = (NumberFormat)formatter.getFormat();
              retVal = format.parse(tf.getText());
            }
          }
        } catch (ParseException e) {
        }
      }
    }
    return retVal;
  }

  /** Sets all associated input controls to their default values. */
  public void resetInput() {
    for (final E key : getControls()) {
      if (key instanceof InputBounds) {
        InputBounds bounds = (InputBounds)key;
        final JFormattedTextField tf = controls.get(key);
        if (tf != null) {
          tf.setText(Integer.toString(bounds.getDefaultValue()));
        }
      }
    }
  }

  /** Sets the preferred size of the input controls to a sane default width. */
  public void setInputDimension() {
    setInputDimension("99999999");
  }

  /** Sets the preferred size of the input controls to the given string length. */
  public void setInputDimension(String prototype) {
    if (prototype == null || prototype.isEmpty()) {
      prototype = "99999999";
    }

    for (final E key : getControls()) {
      final JFormattedTextField tf = controls.get(key);
      if (tf != null) {
        tf.setPreferredSize(Misc.getTextDimension(tf, prototype, true));
      }
    }
  }

  /**
   * Checks whether the input control values conform to their defined bounds.
   *
   * @param ignoreDisabled Specifies whether controls that are disabled should be excluded from the check.
   * @throws ValidationException if a control does not conform to their defined bounds.
   */
  public void validateInput(boolean ignoreDisabled) throws ValidationException {
    for (final E key : getControls()) {
      if (key instanceof InputBounds) {
        InputBounds bounds = (InputBounds)key;
        final JFormattedTextField tf = controls.get(key);
        if (tf != null && (!ignoreDisabled || tf.isEnabled())) {
          final Number value = getValue(key);
          if (value == null) {
            throw new ValidationException("Invalid number entered.", tf);
          } else if (value.doubleValue() < bounds.getMinValue() || value.doubleValue() > bounds.getMaxValue()) {
            final String msg = String.format("Input value is out of range: %s\nValid range: %d - %d", value,
                bounds.getMinValue(), bounds.getMaxValue());
            throw new ValidationException(msg, tf);
          }
        }
      }
    }
  }

  /** Performs an inverse search for the key that is associated with the specified input control. */
  public E getKey(JFormattedTextField tf) {
    for (final Map.Entry<E, JFormattedTextField> entry : controls.entrySet()) {
      if (Objects.equals(entry.getValue(), tf)) {
        return entry.getKey();
      }
    }
    return null;
  }

  // --------------------- Begin Interface KeyListener ---------------------

  @Override
  public void keyTyped(KeyEvent e) {
  }

  @Override
  public void keyPressed(KeyEvent e) {
    if (e.getSource() instanceof JFormattedTextField) {
      final JFormattedTextField tf = (JFormattedTextField)e.getSource();
      final E key = getKey(tf);
      if (!(key instanceof InputBounds)) {
        return;
      }
      final InputBounds bounds = (InputBounds)key;

      // up/down keys increment or decrement current value
      int inc = 0;
      switch (e.getKeyCode()) {
        case KeyEvent.VK_UP:
        case KeyEvent.VK_KP_UP:
          inc = 1;
          break;
        case KeyEvent.VK_DOWN:
        case KeyEvent.VK_KP_DOWN:
          inc = -1;
          break;
        default:
      }

      if (inc != 0) {
        if ((e.getModifiersEx() & KeyEvent.SHIFT_DOWN_MASK) != 0) {
          inc *= 10;
        }

        try {
          if (tf.getFormatter() instanceof NumberFormatter) {
            final NumberFormatter formatter = (NumberFormatter)tf.getFormatter();
            if (formatter.getFormat() instanceof NumberFormat) {
              final NumberFormat format = (NumberFormat)formatter.getFormat();
              // XXX: hack that removes formatting from the number format (works only for Locale-independent number formats)
              final String text = tf.getText().replaceAll(",", "");
              if (format.isParseIntegerOnly()) {
                final int value = Integer.parseInt(text);
                tf.setText(Integer.toString(value + inc));
              } else {
                BigDecimal big = new BigDecimal(text);
                big = big.add(BigDecimal.valueOf(inc, bounds.getScaleValue()));
                tf.setText(big.toPlainString());
              }
              SwingUtilities.invokeLater(tf::selectAll);
            }
          }
        } catch (NumberFormatException ex) {
        }
      }
    }
  }

  @Override
  public void keyReleased(KeyEvent e) {
  }

  // --------------------- End Interface KeyListener ---------------------

  // --------------------- Begin Interface MouseListener ---------------------

  @Override
  public void mouseClicked(MouseEvent e) {
    if (e.getSource() instanceof JFormattedTextField) {
      final JFormattedTextField tf = (JFormattedTextField)e.getSource();
      if (e.getClickCount() == 2) {
        // Invoke later to circumvent content validation (may not work correctly on every platform)
        SwingUtilities.invokeLater(tf::selectAll);
      } else if (!e.isPopupTrigger()) {
        tf.setCaretPosition(tf.viewToModel(e.getPoint()));
      }
    }
  }

  @Override
  public void mousePressed(MouseEvent e) {
  }

  @Override
  public void mouseReleased(MouseEvent e) {
  }

  @Override
  public void mouseEntered(MouseEvent e) {
  }

  @Override
  public void mouseExited(MouseEvent e) {
  }

  // --------------------- End Interface MouseListener ---------------------

  // -------------------------- INNER CLASSES --------------------------

  /** Specialized exception class that is thrown if validation of input controls fails. */
  public static class ValidationException extends Exception {
    private final JFormattedTextField control;

    public ValidationException(String message, JFormattedTextField control) {
      super(message);
      this.control = control;
    }

    /** Returns the associated {@link JFormattedTextField} control. */
    public JFormattedTextField getControl() {
      return control;
    }
  }
}
