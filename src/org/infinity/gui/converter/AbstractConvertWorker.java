// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter;

import java.awt.Component;
import java.beans.PropertyChangeEvent;
import java.beans.PropertyChangeListener;
import java.util.concurrent.ExecutionException;

import javax.swing.ProgressMonitor;
import javax.swing.SwingUtilities;
import javax.swing.SwingWorker;

import org.infinity.NearInfinity;
import org.infinity.gui.WindowBlocker;
import org.infinity.util.Misc;
import org.tinylog.Logger;

public abstract class AbstractConvertWorker<T> extends SwingWorker<T, Void> implements PropertyChangeListener {
  private final Component parent;

  private WindowBlocker blocker;
  private ProgressMonitor progress;
  private int minProgress, maxProgress;
  private volatile int curProgress;
  private int decideDelay, popupDelay;
  private String progressMessage;
  private volatile String progressNote;
  private boolean isBlocking, showProgress;
  private volatile boolean isWorking;

  /**
   * Helper method that rescales the specified {@code percent} value into a range of 0 to {@code size}.
   *
   * @param percent Percent value between 0 and 100 to rescale.
   * @param range   Upper bounds of the range to rescale.
   * @return Value scaled to the given range.
   */
  public static int getScaledValue(int percent, int range) {
    percent = Misc.clamp(percent, 0, 100);
    return percent * range / 100;
  }

  /**
   * Helper method that normalizes the specified {@code value} into percentage range (0 to 100).
   *
   * @param value Value to scale into percent range (0-100).
   * @param range Upper bounds of the source range.
   * @return Normalized percent value.
   */
  public static int getPercentValue(int value, int range) {
    if (range == 0) {
      return 0;
    }
    value = Misc.clamp(value, 0, range);
    return value * 100 / range;
  }

  /**
   * Initializes the worker instance with default parameters.
   *
   * @param parent Parent {@link Component} that is used by blocker and progress monitor.
   */
  protected AbstractConvertWorker(Component parent) {
    this(parent, true, false, 0, 100, "", 250, 2000);
  }

  /**
   * Initializes the worker instance with the specified parameters.
   *
   * @param parent          Parent {@link Component} that is used by blocker and progress monitor.
   * @param block           Specifies whether parent components should be blocked from user access when a task is
   *                          running.
   * @param showProgress    Specifies whether a progress monitor should be shown when a task is running.
   * @param minProgress     Sets the lower bounds of the progress range.
   * @param maxProgress     Sets the upper bounds of the progress range.
   * @param progressMessage Message displayed in the progress monitor dialog.
   * @param decideDelay     Amount of time (milliseconds) before the progress monitor decides whether to pop up.
   * @param popupDelay      Amount of time (milliseconds) before the progress monitor pops up.
   */
  protected AbstractConvertWorker(Component parent, boolean block, boolean showProgress, int minProgress,
      int maxProgress, String progressMessage, int decideDelay, int popupDelay) {
    super();
    this.parent = parent;
    this.curProgress = 0;
    this.isWorking = false;
    setBlockingEnabled(block);
    setShowProgressEnabled(showProgress);
    setMinProgress(minProgress);
    setMaxProgress(maxProgress);
    setProgressMessage(progressMessage);
    setProgressNote("");
    setDecideDelay(decideDelay);
    setPopupDelay(popupDelay);
    addPropertyChangeListener(this);
  }

  /** Returns whether the parent window should be blocked while a task is active. */
  public boolean isBlockingEnabled() {
    return isBlocking;
  }

  /**
   * Sets whether the parent window should be blocked while a task is active.
   * <p>
   * Note: Can only be set while the worker is not active.
   * </p>
   */
  public void setBlockingEnabled(boolean enable) {
    if (!isWorking()) {
      isBlocking = enable;
    }
  }

  /** Returns whether a progress monitor should be shown while the task is active. */
  public boolean isShowProgressEnabled() {
    return showProgress;
  }

  /**
   * Sets whether a progress monitor should be shown while the task is active.
   * <p>
   * Note: Can only be set while the worker is not active.
   * </p>
   */
  public void setShowProgressEnabled(boolean show) {
    if (!isWorking()) {
      showProgress = show;
    }
  }

  /** Advances progress by one percent point. */
  public void advanceProgress() {
    advanceProgress(1);
  }

  /** Advances progress by the specified amount of percent points. */
  public void advanceProgress(int steps) {
    advanceProgressTo(curProgress + steps);
  }

  /** Sets current progress to the specified value. */
  public void advanceProgressTo(int newValue) {
    if (isWorking()) {
      newValue = Misc.clamp(newValue, curProgress, maxProgress);
      if (newValue != curProgress) {
        curProgress = newValue;
        if (progress != null) {
          if (isWorking() && !SwingUtilities.isEventDispatchThread()) {
            SwingUtilities.invokeLater(() -> progress.setProgress(curProgress));
          } else {
            progress.setProgress(curProgress);
          }
        }
        setProgress(getPercentValue(curProgress, maxProgress - minProgress));
      }
    }
  }

  /** Returns the current progress value, in percent. */
  public int getCurrentProgress() {
    return curProgress;
  }

  /** Returns the lower bounds of the progress range. */
  public int getMinProgress() {
    return minProgress;
  }

  /**
   * Sets the lower bounds of the progress range.
   * <p>
   * Note: Can only be set while the worker is not active.
   * </p>
   */
  public void setMinProgress(int newValue) {
    if (!isWorking() && newValue != minProgress) {
      minProgress = Math.max(0, newValue);
      setMaxProgress(maxProgress);
    }
  }

  /** Returns the upper bounds of the progress range. */
  public int getMaxProgress() {
    return maxProgress;
  }

  /**
   * Sets the upper bounds of the progress range.
   * <p>
   * Note: Can only be set while the worker is not active.
   * </p>
   */
  public void setMaxProgress(int newValue) {
    if (!isWorking()) {
      maxProgress = Math.max(minProgress + 1, newValue);
    }
  }

  /** Returns the message {@code String} associated with the progress monitor. */
  public String getProgressMessage() {
    return progressMessage;
  }

  /**
   * Sets the message {@code String} associated with the progress monitor.
   * <p>
   * Note: Can only be set while the worker is not active.
   * </p>
   */
  public void setProgressMessage(String message) {
    if (!isWorking()) {
      progressMessage = (message != null) ? message : "";
    }
  }

  /** Returns the current note {@code String} associated with the progress monitor. */
  public String getProgressNote() {
    return progressNote;
  }

  /**
   * Sets the current note {@code String} associated with the progress monitor. The note can be updated while a task
   * is in progress.
   */
  public void setProgressNote(String note) {
    progressNote = (note != null) ? note : "";
    if (progress != null) {
      if (isWorking()) {
        SwingUtilities.invokeLater(() -> progress.setNote(progressNote));
      } else {
        progress.setNote(progressNote);
      }
    }
  }

  /** Returns the amount of time (in milliseconds) before deciding whether to show a progress monitor. */
  public int getDecideDelay() {
    return decideDelay;
  }

  /**
   * Sets the amount of time (in milliseconds) before deciding whether to show a progress monitor.
   * <p>
   * Note: Can only be set while the worker is not active.
   * </p>
   */
  public void setDecideDelay(int newValue) {
    if (!isWorking()) {
      decideDelay = Math.max(0, newValue);
      setPopupDelay(popupDelay);
    }
  }

  /** Returns the amount of time (in milliseconds) before the progress monitor is shown. */
  public int getPopupDelay() {
    return popupDelay;
  }

  /**
   * Sets the amount of time (in milliseconds) before the progress monitor is shown.
   * <p>
   * Note: Can only be set while the worker is not active.
   * </p>
   */
  public void setPopupDelay(int newValue) {
    if (!isWorking()) {
      popupDelay = Math.max(decideDelay, newValue);
    }
  }

  /**
   * Returns whether the user signaled to cancel the process. This option is only available when the worker is active
   * and the progress monitor is visible.
   *
   * @return {@code true} if progress was cancelled by the user, {@code false} otherwise.
   */
  public boolean isProgressCancelled() {
    if (isWorking() && progress != null) {
      return progress.isCanceled();
    }
    return false;
  }

  /** Returns whether the worker is currently active. */
  public boolean isWorking() {
    return isWorking;
  }

  // --------------------- Begin Interface PropertyChangeListener ---------------------

  @Override
  public void propertyChange(PropertyChangeEvent evt) {
    if ("progress".equals(evt.getPropertyName())) {
      final int progressValue = (Integer)evt.getNewValue();
      onProgress(progressValue);
    } else if ("state".equals(evt.getPropertyName())) {
      final StateValue curState = (StateValue)evt.getNewValue();
      switch (curState) {
        case STARTED:
          if (!onStarted()) {
            cancel(false);
          }
          init();
          isWorking = true;
          break;
        case DONE:
          isWorking = false;
          advanceProgressTo(maxProgress);
          cleanup();

          if (isCancelled()) {
            onCancelled();
          } else {
            T result;
            try {
              result = get();
            } catch (InterruptedException | ExecutionException e) {
              result = null;
              Logger.error(e);
            }
            onCompleted(result);
          }
          break;
        default:
      }
    }
  }

  // --------------------- End Interface PropertyChangeListener ---------------------

  /**
   * <p>
   * <strong>Note:</strong> This method must be overridden by the derived class to perform the background task.
   * </p>
   *
   * {@inheritDoc}
   */
  @Override
  protected abstract T doInBackground() throws Exception;

  /**
   * Called when the task is completed. This method must be implemented by derived classes.
   *
   * @param result the return value of the background task.
   */
  protected abstract void onCompleted(T result);

  /**
   * Called when the task is terminated prematurely. Does nothing in the default implementation.
   */
  protected void onCancelled() {
  }

  /**
   * Called right before the background task is invoked. Does nothing in the default implementation.
   *
   * @return {@code true} if the worker should continue with the task, {@code false} if the task should be cancelled.
   */
  protected boolean onStarted() {
    return true;
  }

  /**
   * Called whenever the task progress advances. Does nothing in the default implementation.
   *
   * @param progress Current progress (as percent value between 0 and 100).
   */
  protected void onProgress(int progress) {
  }

  /** Performs initializations when a task is about to be started. */
  private void init() {
    if (isBlocking) {
      blocker = (parent != null)
          ? new WindowBlocker(WindowBlocker.getRootPaneAncestor(parent, NearInfinity.getInstance()))
          : null;
      blocker.setBlocked(true);
    }

    if (showProgress) {
      progress = new ProgressMonitor(parent, progressMessage, progressNote, minProgress, maxProgress);
      progress.setMillisToDecideToPopup(decideDelay);
      progress.setMillisToPopup(popupDelay);
    }
  }

  /** Performs some internal cleanups after the task is completed. */
  private void cleanup() {
    if (blocker != null) {
      blocker.setBlocked(false);
      blocker = null;
    }

    if (progress != null) {
      progress.close();
      progress = null;
    }
  }
}