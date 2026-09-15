// # 📄 Dosya Yolu: /turkuazvm/guest/android-agent/src/com/turkuazvm/inputagent/tools/TouchGestureTool.kt
// # 📌 Amac: Normalized persistent touch frame'lerini Android Accessibility gesture stroke'larina adapte eder
// # 📌 Modul - Kotlin
// # Version: 0.12.4
// # Aciklama: Stroke continuation state'ini pointer bazinda korur ve multi-touch frame'i tek GestureDescription olarak dispatch eder
// # Bagimli Oldugu Katman: Service | Tool

package com.turkuazvm.inputagent.tools

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.graphics.Path
import android.os.Handler
import android.os.Looper
import android.util.DisplayMetrics
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

private const val NORMALIZED_MAX = 10_000.0f
private const val CALLBACK_TIMEOUT_MS = 250L

data class AgentTouchContact(
    val pointerId: Int,
    val phase: String,
    val x: Int,
    val y: Int,
)

private data class ActiveStroke(
    val stroke: GestureDescription.StrokeDescription,
    val x: Float,
    val y: Float,
)

class TouchGestureTool(
    private val service: AccessibilityService,
    private val maxContacts: Int,
    private val frameDurationMs: Long,
) {
    private val active = linkedMapOf<Int, ActiveStroke>()
    private val mainHandler = Handler(Looper.getMainLooper())

    @Synchronized
    fun applyFrame(contacts: List<AgentTouchContact>): Result<Unit> {
        if (contacts.size > maxContacts) {
            return Result.failure(IllegalArgumentException("contact_limit_exceeded"))
        }
        val duplicatePointer = contacts.groupingBy { it.pointerId }.eachCount().any { it.value > 1 }
        if (duplicatePointer) {
            return Result.failure(IllegalArgumentException("duplicate_pointer_id"))
        }
        if (contacts.any { it.x !in 0..10_000 || it.y !in 0..10_000 || it.pointerId !in 0..255 }) {
            return Result.failure(IllegalArgumentException("invalid_touch_contact"))
        }

        val metrics = service.resources.displayMetrics
        val builder = GestureDescription.Builder()
        val nextActive = linkedMapOf<Int, ActiveStroke>()

        for (contact in contacts) {
            val point = toPixel(contact, metrics)
            val existing = active[contact.pointerId]
            val stroke = when (contact.phase) {
                "down" -> newStroke(point.first, point.second, willContinue = true)
                "move" -> existing?.let {
                    continueStroke(it, point.first, point.second, willContinue = true)
                } ?: newStroke(point.first, point.second, willContinue = true)
                "up" -> existing?.let {
                    continueStroke(it, point.first, point.second, willContinue = false)
                } ?: newStroke(point.first, point.second, willContinue = false)
                else -> return Result.failure(IllegalArgumentException("invalid_touch_phase"))
            }
            builder.addStroke(stroke)
            if (contact.phase != "up") {
                nextActive[contact.pointerId] = ActiveStroke(stroke, point.first, point.second)
            }
        }

        if (contacts.isEmpty()) {
            active.clear()
            return Result.success(Unit)
        }
        val dispatched = dispatch(builder.build())
        if (!dispatched) {
            return Result.failure(IllegalStateException("gesture_dispatch_failed"))
        }
        active.clear()
        active.putAll(nextActive)
        return Result.success(Unit)
    }

    @Synchronized
    fun reset(): Result<Unit> {
        if (active.isEmpty()) {
            return Result.success(Unit)
        }
        val builder = GestureDescription.Builder()
        active.values.forEach { state ->
            builder.addStroke(continueStroke(state, state.x, state.y, willContinue = false))
        }
        val dispatched = dispatch(builder.build())
        active.clear()
        return if (dispatched) Result.success(Unit)
        else Result.failure(IllegalStateException("gesture_reset_failed"))
    }

    private fun newStroke(x: Float, y: Float, willContinue: Boolean): GestureDescription.StrokeDescription {
        val path = Path().apply { moveTo(x, y) }
        return GestureDescription.StrokeDescription(path, 0L, frameDurationMs, willContinue)
    }

    private fun continueStroke(
        activeStroke: ActiveStroke,
        x: Float,
        y: Float,
        willContinue: Boolean,
    ): GestureDescription.StrokeDescription {
        val path = Path().apply {
            moveTo(activeStroke.x, activeStroke.y)
            lineTo(x, y)
        }
        return activeStroke.stroke.continueStroke(path, 0L, frameDurationMs, willContinue)
    }

    private fun toPixel(contact: AgentTouchContact, metrics: DisplayMetrics): Pair<Float, Float> {
        val width = metrics.widthPixels.coerceAtLeast(1).toFloat()
        val height = metrics.heightPixels.coerceAtLeast(1).toFloat()
        val x = contact.x.toFloat() / NORMALIZED_MAX * (width - 1.0f)
        val y = contact.y.toFloat() / NORMALIZED_MAX * (height - 1.0f)
        return Pair(x, y)
    }

    private fun dispatch(gesture: GestureDescription): Boolean {
        val latch = CountDownLatch(1)
        var completed = false
        mainHandler.post {
            val accepted = service.dispatchGesture(
                gesture,
                object : AccessibilityService.GestureResultCallback() {
                    override fun onCompleted(gestureDescription: GestureDescription?) {
                        completed = true
                        latch.countDown()
                    }

                    override fun onCancelled(gestureDescription: GestureDescription?) {
                        completed = false
                        latch.countDown()
                    }
                },
                mainHandler,
            )
            if (!accepted) {
                completed = false
                latch.countDown()
            }
        }
        return latch.await(CALLBACK_TIMEOUT_MS, TimeUnit.MILLISECONDS) && completed
    }
}
