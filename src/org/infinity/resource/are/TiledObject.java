// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.are;

import java.nio.ByteBuffer;

import org.infinity.datatype.DecNumber;
import org.infinity.datatype.Flag;
import org.infinity.datatype.IsNumeric;
import org.infinity.datatype.SectionCount;
import org.infinity.datatype.TextString;
import org.infinity.datatype.Unknown;
import org.infinity.resource.AbstractStruct;
import org.infinity.resource.AddRemovable;
import org.infinity.resource.HasChildStructs;
import org.infinity.resource.StructEntry;
import org.infinity.resource.vertex.ClosedVertexTiled;
import org.infinity.resource.vertex.OpenVertexTiled;
import org.infinity.resource.vertex.Vertex;
import org.infinity.util.io.StreamUtils;

public final class TiledObject extends AbstractStruct implements AddRemovable, HasVertices, HasChildStructs {
  // ARE/Tiled Object-specific field labels
  public static final String ARE_TILED                            = "Tiled object";
  public static final String ARE_TILED_NAME                       = "Name";
  public static final String ARE_TILED_ID                         = "Tile ID";
  public static final String ARE_TILED_FLAGS                      = "Tile flags";
  public static final String ARE_TILED_FIRST_SQUARE_INDEX_OPEN    = "First search square index (open)";
  public static final String ARE_TILED_FIRST_SQUARE_INDEX_CLOSED  = "First search square index (closed)";
  public static final String ARE_TILED_NUM_SQUARES_OPEN           = "# search squares (open)";
  public static final String ARE_TILED_NUM_SQUARES_CLOSED         = "# search squares (closed)";

  public static final String[] FLAGS_ARRAY = { "No flags set", "Closed tile", "Can be looked through" };

  TiledObject() throws Exception {
    super(null, ARE_TILED, StreamUtils.getByteBuffer(104), 0);
  }

  TiledObject(AbstractStruct superStruct, ByteBuffer buffer, int offset, int number) throws Exception {
    super(superStruct, ARE_TILED + " " + number, buffer, offset);
  }

  // --------------------- Begin Interface HasChildStructs ---------------------

  @Override
  public AddRemovable[] getPrototypes() throws Exception {
    return new AddRemovable[] { new OpenVertexTiled(), new ClosedVertexTiled() };
  }

  @Override
  public AddRemovable confirmAddEntry(AddRemovable entry) throws Exception {
    return entry;
  }

  // --------------------- End Interface HasChildStructs ---------------------

  // --------------------- Begin Interface AddRemovable ---------------------

  @Override
  public boolean canRemove() {
    return true;
  }

  // --------------------- End Interface AddRemovable ---------------------

  // --------------------- Begin Interface HasVertices ---------------------

  @Override
  public void readVertices(ByteBuffer buffer, int offset) throws Exception {
    IsNumeric firstVertex = (IsNumeric) getAttribute(ARE_TILED_FIRST_SQUARE_INDEX_OPEN);
    IsNumeric numVertices = (IsNumeric) getAttribute(ARE_TILED_NUM_SQUARES_OPEN);
    for (int i = 0; i < numVertices.getValue(); i++) {
      addField(new OpenVertexTiled(this, buffer, offset + 4 * (firstVertex.getValue() + i), i));
    }

    firstVertex = (IsNumeric) getAttribute(ARE_TILED_FIRST_SQUARE_INDEX_CLOSED);
    numVertices = (IsNumeric) getAttribute(ARE_TILED_NUM_SQUARES_CLOSED);
    for (int i = 0; i < numVertices.getValue(); i++) {
      addField(new ClosedVertexTiled(this, buffer, offset + 4 * (firstVertex.getValue() + i), i));
    }
  }

  @Override
  public int updateVertices(int offset, int number) {
    ((DecNumber) getAttribute(ARE_TILED_FIRST_SQUARE_INDEX_OPEN)).setValue(number);
    int count = ((IsNumeric) getAttribute(ARE_TILED_NUM_SQUARES_OPEN)).getValue();
    ((DecNumber) getAttribute(ARE_TILED_FIRST_SQUARE_INDEX_CLOSED)).setValue(number + count);
    count += ((IsNumeric) getAttribute(ARE_TILED_NUM_SQUARES_CLOSED)).getValue();

    for (final StructEntry entry : getFields()) {
      if (entry instanceof Vertex) {
        entry.setOffset(offset);
        ((Vertex) entry).realignStructOffsets();
        offset += 4;
      }
    }
    return count;
  }

  // --------------------- End Interface HasVertices ---------------------

  @Override
  protected void setAddRemovableOffset(AddRemovable datatype) {
    final int offset = ((IsNumeric) getParent().getAttribute(AreResource.ARE_OFFSET_VERTICES)).getValue();
    if (datatype instanceof OpenVertexTiled) {
      int index = ((IsNumeric) getAttribute(ARE_TILED_FIRST_SQUARE_INDEX_OPEN)).getValue();
      index += ((IsNumeric) getAttribute(ARE_TILED_NUM_SQUARES_OPEN)).getValue();
      datatype.setOffset(offset + 4 * (index - 1));
    } else if (datatype instanceof ClosedVertexTiled) {
      int index = ((IsNumeric) getAttribute(ARE_TILED_FIRST_SQUARE_INDEX_CLOSED)).getValue();
      index += ((IsNumeric) getAttribute(ARE_TILED_NUM_SQUARES_CLOSED)).getValue();
      datatype.setOffset(offset + 4 * (index - 1));
    }
  }

  @Override
  public int read(ByteBuffer buffer, int offset) throws Exception {
    addField(new TextString(buffer, offset, 32, ARE_TILED_NAME));
    addField(new TextString(buffer, offset + 32, 8, ARE_TILED_ID));
    addField(new Flag(buffer, offset + 40, 4, ARE_TILED_FLAGS, FLAGS_ARRAY));
    addField(new DecNumber(buffer, offset + 44, 4, ARE_TILED_FIRST_SQUARE_INDEX_OPEN));
    addField(new SectionCount(buffer, offset + 48, 2, ARE_TILED_NUM_SQUARES_OPEN, OpenVertexTiled.class));
    addField(new SectionCount(buffer, offset + 50, 2, ARE_TILED_NUM_SQUARES_CLOSED, ClosedVertexTiled.class));
    addField(new DecNumber(buffer, offset + 52, 4, ARE_TILED_FIRST_SQUARE_INDEX_CLOSED));
    addField(new Unknown(buffer, offset + 56, 48));
    return offset + 104;
  }
}
