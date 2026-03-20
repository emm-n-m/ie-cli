// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.EventObject;
import java.util.List;

/**
 * PanelUpdateEvent is used to inform registered listeners about a configuration change in the event source.
 * More specific information can be queried through the associated update reason.
 */
public class PanelUpdateEvent extends EventObject {
  private static final long serialVersionUID = 1L | (long)PanelUpdateEvent.class.hashCode() << 32;

  /** A generic {@link UpdateReason} that is used if no specific reason can be defined. */
  public static final UpdateReason REASON_UNKNOWN = new UpdateReason(-1);

  private final UpdateReason reason;
  private final List<Object> parameters;

  /**
   * Constructs a new {@code PanelUpdateEvent} object.
   *
   * @param source The object on which the Event initially occurred.
   * @param reason Reason for this update.
   * @exception IllegalArgumentException if source is null.
   */
  public PanelUpdateEvent(Object source, UpdateReason reason) {
    this(source, reason, null);
  }

  /**
   * Constructs a new {@code PanelUpdateEvent} object.
   *
   * @param source     The object on which the Event initially occurred.
   * @param reason     Reason for this update.
   * @param parameters Optional list of parameters that are associated with the specified {@code reason}.
   * @exception IllegalArgumentException if source is null.
   */
  public PanelUpdateEvent(Object source, UpdateReason reason, Collection<Object> parameters) {
    super(source);
    this.reason = (reason != null) ? reason : REASON_UNKNOWN;
    if (parameters != null) {
      this.parameters = Collections.unmodifiableList(new ArrayList<>(parameters));
    } else {
      this.parameters = Collections.unmodifiableList(new ArrayList<>());
    }
  }

  /** Returns the {@link UpdateReason} for this event. */
  public UpdateReason getReason() {
    return reason;
  }

  /**
   * Returns a list of optional parameters that are associated with the event. Parameters depend on the specific
   * reason and can be of any type.
   *
   * @return Unmodifiable list of parameters. The list may be empty but is never {@code null}.
   */
  public List<Object> getParameters() {
    return parameters;
  }

  @Override
  public String toString() {
    StringBuilder builder = new StringBuilder();
    builder.append("PanelUpdateEvent [source=").append(source).append(", reason=").append(reason)
        .append(", parameters=").append(parameters).append("]");
    return builder.toString();
  }
}