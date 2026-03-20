// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bam;

import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.MouseEvent;
import java.awt.event.MouseListener;

import javax.swing.AbstractButton;
import javax.swing.Timer;

import org.infinity.util.Operation;

/**
 * Handles delayed execution of user-defined operations action events from button controls.
 */
class DelayedButtonAction implements ActionListener, MouseListener {
  private final Timer actionTimer;

  private AbstractButton button;
  private Operation operation;

  public DelayedButtonAction() {
    this.actionTimer = new Timer(1 << 31, null);
    this.actionTimer.stop();
    this.actionTimer.setRepeats(false);
  }

  /**
   * <p>
   * Sets up a delayed operation. The operation is executed in the event-dispatching thread. The original action event
   * that is fired if the specified button is released will be temporarily suppressed when the delayed operation is
   * performed.
   * </p>
   * <p>
   * Setting up a new delayed action will terminate any previously defined delayed actions.
   * </p>
   *
   * @param button {@link AbstractButton} instance of the button control that performs the delayed operation.
   * @param delay  Delay in milliseconds.
   * @param op     {@link Operation} to perform after the specified delay. Specifying {@code null} still fires at the
   *                 specified delay but doesn't perform any operations.
   */
  public synchronized void performDelayedAction(AbstractButton button, int delay, Operation op) {
    resetDelayedAction();

    this.button = button;
    if (this.button != null) {
      // we also have to deal with cancelled long-taps
      this.button.addMouseListener(this);
    }
    operation = op;

    actionTimer.addActionListener(e -> {
      // current button click is cancelled to prevent firing action events twice
      if (this.button != null) {
        this.button.getModel().setArmed(false);
        this.button.getModel().setPressed(false);
        this.button.getModel().setArmed(true);
      }
      resetDelayedAction();
      if (op != null) {
        op.perform();
      }
    });

    actionTimer.setInitialDelay(Math.max(0, delay));
    actionTimer.start();
  }

  /**
   * Terminates an ongoing delayed action that was set up by
   * {@link #performDelayedAction(AbstractButton, int, Operation)}.
   */
  public synchronized void resetDelayedAction() {
    if (actionTimer.isRunning()) {
      actionTimer.stop();
    }

    operation = null;
    if (button != null) {
      button.removeMouseListener(this);
      button = null;
    }

    // removing active listeners
    final ActionListener[] listeners = actionTimer.getActionListeners();
    for (int i = listeners.length - 1; i >= 0; i--) {
      actionTimer.removeActionListener(listeners[i]);
    }
  }

  @Override
  public void actionPerformed(ActionEvent e) {
    final AbstractButton button = this.button;
    final Operation op = this.operation;

    resetDelayedAction();

    // current button click is cancelled to prevent firing action events multiple times
    if (button != null) {
      button.getModel().setArmed(false);
      button.getModel().setPressed(false);
      button.getModel().setArmed(true);
    }

    if (op != null) {
      op.perform();
    }
  }

  @Override
  public void mouseClicked(MouseEvent e) {
  }

  @Override
  public void mousePressed(MouseEvent e) {
  }

  @Override
  public void mouseReleased(MouseEvent e) {
    resetDelayedAction();
  }

  @Override
  public void mouseEntered(MouseEvent e) {
  }

  @Override
  public void mouseExited(MouseEvent e) {
    resetDelayedAction();
  }
}