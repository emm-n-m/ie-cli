// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.vertex;

import java.nio.ByteBuffer;

import org.infinity.resource.AbstractStruct;
import org.infinity.util.io.StreamUtils;

public final class ClosedVertexTiled extends Vertex {
  // ClosedVertexTiled-specific field labels
  public static final String VERTEX_CLOSED_SEARCH = "Search square (closed)";

  public ClosedVertexTiled() throws Exception {
    super(null, VERTEX_CLOSED_SEARCH, StreamUtils.getByteBuffer(4), 0);
  }

  public ClosedVertexTiled(AbstractStruct superStruct, ByteBuffer buffer, int offset, int nr) throws Exception {
    super(superStruct, VERTEX_CLOSED_SEARCH + " " + nr, buffer, offset);
  }
}
