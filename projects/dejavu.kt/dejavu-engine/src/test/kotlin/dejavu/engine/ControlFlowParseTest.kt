package dejavu.engine

import dejavu.Dejavu
import dejavu.language.ParseError
import kotlinx.serialization.json.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class ControlFlowParseTest {
    private fun firstLoopBody(ir: JsonObject): JsonArray =
        ir["body"]!!.jsonObject["children"]!!.jsonArray[0].jsonObject["body"]!!.jsonArray

    @Test
    fun controlStatementsInsideLoopLowerToNodes() {
        val breakIr = Dejavu.parse("<% loop x in xs %><% break %><% end loop %>")
        val breakBody = firstLoopBody(breakIr)
        assertEquals("Stmt.Break", breakBody[0].jsonObject["type"]!!.jsonPrimitive.content)

        val continueIr = Dejavu.parse("<% loop x in xs %><% continue %><% end loop %>")
        val continueBody = firstLoopBody(continueIr)
        assertEquals("Stmt.Continue", continueBody[0].jsonObject["type"]!!.jsonPrimitive.content)
    }

    @Test
    fun controlStatementsRenderFromSource() {
        val ctx = buildJsonObject {
            put("xs", buildJsonArray { add(1); add(2); add(3) })
        }
        assertEquals(
            "1",
            Dejavu.renderSource(
                "<% loop x in xs %><% if x == 2 %><% break %><% end if %><% x %><% end loop %>",
                ctx,
            ),
        )
        assertEquals(
            "13",
            Dejavu.renderSource(
                "<% loop x in xs %><% if x == 2 %><% continue %><% end if %><% x %><% end loop %>",
                ctx,
            ),
        )
    }

    @Test
    fun breakInNestedLoopBindsTheInnerLoop() {
        val ctx = buildJsonObject {
            put("xs", buildJsonArray { add(1); add(2) })
            put("ys", buildJsonArray { add(9) })
        }
        assertEquals(
            "12",
            Dejavu.renderSource(
                "<% loop a in xs %><% loop b in ys %><% break %><% end loop %><% a %><% end loop %>",
                ctx,
            ),
        )
    }

    @Test
    fun controlStatementOutsideLoopIsAParseError() {
        for ((source, kind) in listOf("<% break %>" to "break", "<% continue %>" to "continue")) {
            val err = assertFailsWith<ParseError> { Dejavu.parse(source) }
            assertEquals("dejavu.parse.control-outside-loop", err.code)
            assertEquals(kind, err.diagnosticArgs["kind"])
        }

        val inIf = assertFailsWith<ParseError> {
            Dejavu.parse("<% if true %><% break %><% end if %>")
        }
        assertEquals("dejavu.parse.control-outside-loop", inIf.code)
        assertEquals("break", inIf.diagnosticArgs["kind"])
    }
}
