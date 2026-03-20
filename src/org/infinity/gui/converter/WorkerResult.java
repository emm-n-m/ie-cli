// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.List;

/**
 * Encapsulates the result state of a background task.
 */
public class WorkerResult {
  private final List<Object> parameters = new ArrayList<>();
  private final boolean success;
  private final String message;

  /**
   * Initializes a new {@code WorkerResult} object.
   *
   * @param isSuccess  Specifies whether the operation was successful.
   */
  public WorkerResult(boolean isSuccess) {
    this(isSuccess, null, null);
  }

  /**
   * Initializes a new {@code WorkerResult} object.
   *
   * @param isSuccess  Specifies whether the operation was successful.
   * @param message    A descriptive message associated with the success state.
   */
  public WorkerResult(boolean isSuccess, String message) {
    this(isSuccess, message, null);
  }

  /**
   * Initializes a new {@code WorkerResult} object.
   *
   * @param isSuccess  Specifies whether the operation was successful.
   * @param message    A descriptive message associated with the success state.
   * @param parameters Optional list of parameters with further information.
   */
  public WorkerResult(boolean isSuccess, String message, Collection<Object> parameters) {
    this.success = isSuccess;
    this.message = (message != null) ? message : "";
    if (parameters != null) {
      this.parameters.addAll(parameters);
    }
  }

  /** Returns whether the result is considered a success. */
  public boolean isSuccess() {
    return success;
  }

  /** Returns a message string that is associated with this result object. */
  public String getMessage() {
    return message;
  }

  /** Returns an unmodifiable list with associated parameters. */
  public List<Object> getParameters() {
    return Collections.unmodifiableList(parameters);
  }

  @Override
  public String toString() {
    StringBuilder builder = new StringBuilder();
    builder.append("WorkerResult [success=").append(success).append(", message=").append(message)
        .append(", parameters=").append(parameters).append("]");
    return builder.toString();
  }
}