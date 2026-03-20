// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bam;

import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.KeyAdapter;
import java.awt.event.KeyEvent;

import javax.swing.InputVerifier;
import javax.swing.JComponent;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.JTextField;
import javax.swing.event.EventListenerList;
import javax.swing.text.JTextComponent;

import org.infinity.gui.ViewerUtil;

/**
 * A custom panel for jumping directly to a specific cycle index in the preview tab.
 */
// TODO
class PreviewGotoCyclePanel extends JPanel {
  public static final String ACTION_ACCEPT = "VALUE_ACCEPTED";
  public static final String ACTION_DISCARD = "VALUE_DISCARDED";

  private final InputVerifier inputVerifier = new InputVerifier() {
    @Override
    public boolean verify(JComponent input) {
      if (input instanceof JTextComponent) {
        final JTextComponent tc = (JTextComponent)input;
        if (!tc.getText().isEmpty()) {
          try {
            Integer.parseInt(tc.getText());
          } catch (NumberFormatException e) {
            return false;
          }
        }
      }
      return true;
    }
  };

  private final EventListenerList listenerList = new EventListenerList();
  private final JLabel label = new JLabel("Go to:");
  private final JTextField cycleInput = new JTextField(5);

  private int defValue;

  public PreviewGotoCyclePanel() {
    super(new GridBagLayout());
    init();
  }

  /** Returns the default value that is used if the user entered an invalid number. */
  public int getDefaultValue() {
    return defValue;
  }

  /** Assigns a new default value to the panel that is used if the user entered an invalid number. */
  public void setDefaultValue(int defaultValue) {
    defValue = defaultValue;
  }

  /**
   * Returns the numeric representation of the input field content. Returns the assigned default value if content
   * could not be parsed.
   */
  public int getValue() {
    return getValue(getDefaultValue());
  }

  /**
   * Returns the numeric representation of the input field content. Returns {@code defValue} if content could not be
   * parsed.
   */
  public int getValue(int defValue) {
    try {
      return Integer.parseInt(cycleInput.getText());
    } catch (NumberFormatException e) {
      return defValue;
    }
  }

  /** Assigns a new value to the input field. */
  public void setValue(int newValue) {
    cycleInput.setText(Integer.toString(newValue));
  }

  /** Sets focus to the input field and selects all content. */
  public void activate() {
    cycleInput.selectAll();
    cycleInput.requestFocusInWindow();
  }

  /**
   * Registers the specified action listener. An action event is fired if the user presses {@code ENTER} while the
   * input field has the focus.
   */
  public void addActionListener(ActionListener l) {
    if (l != null) {
      listenerList.add(ActionListener.class, l);
    }
  }

  /**
   * Unregisters the specified action listener. An action event is fired if the user presses {@code ENTER} while the
   * input field has the focus.
   */
  public void removeActionListener(ActionListener l) {
    if ((l != null)) {
      listenerList.remove(ActionListener.class, l);
    }
  }

  /** Returns a list of all registered action listeners. */
  public ActionListener[] getActionListeners() {
    return listenerList.getListeners(ActionListener.class);
  }

  /** Fires an action event to all registered listeners. */
  protected void fireActionPerformed(String actionCommand) {
    Object[] listeners = listenerList.getListenerList();
    ActionEvent e = null;
    for (int i = listeners.length - 2; i >= 0; i -= 2) {
      if (listeners[i] == ActionListener.class) {
        if (e == null) {
          e = new ActionEvent(PreviewGotoCyclePanel.this, ActionEvent.ACTION_PERFORMED, actionCommand,
              System.currentTimeMillis(), 0);
        }
        ((ActionListener)listeners[i + 1]).actionPerformed(e);
      }
    }
  }

  /** Initializes UI components. */
  private void init() {
    cycleInput.setToolTipText("Change value with UP/DOWN keys. Press SHIFT key to increase step size.");
    cycleInput.setInputVerifier(inputVerifier);
    cycleInput.addKeyListener(new KeyAdapter() {
      @Override
      public void keyPressed(KeyEvent e) {
        switch (e.getKeyCode()) {
          case KeyEvent.VK_ENTER:
            fireActionPerformed(ACTION_ACCEPT);
            break;
          case KeyEvent.VK_ESCAPE:
            fireActionPerformed(ACTION_DISCARD);
            break;
          case KeyEvent.VK_UP:
          {
            final int step = (e.getModifiersEx() & KeyEvent.SHIFT_DOWN_MASK) != 0 ? 10 : 1;
            setValue(getValue() + step);
            activate();
            break;
          }
          case KeyEvent.VK_DOWN:
            final int step = (e.getModifiersEx() & KeyEvent.SHIFT_DOWN_MASK) != 0 ? 10 : 1;
            setValue(Math.max(0, getValue() - step));
            activate();
            break;
          default:
        }
      }
    });

    GridBagConstraints c = new GridBagConstraints();
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(4, 4, 4, 0), 0, 0);
    add(label, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(4, 4, 4, 4), 0, 0);
    add(cycleInput, c);
  }
}